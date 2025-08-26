use crate::ast::*;
use crate::error::EvaluationError;
use std::collections::{HashMap, HashSet};

/// Holds the compiled, optimized ASTs and is ready for fast evaluation.
pub struct Evaluator {
    // For this POC, we'll just hold all paths. A real system
    // would split them into static and dynamic.
    pub quality_paths: Vec<(i32, String, Expression)>,
}

/// The final result of an evaluation.
#[derive(Debug)]
pub struct EvaluationResult {
    pub quality_name: Option<String>,
    pub quality_priority: Option<i32>,
    // In the future, this could be the EvaluationTrace
    pub reason: String,
}

impl Evaluator {
    pub fn new(quality_paths: Vec<(i32, String, Expression)>) -> Self {
        Self { quality_paths }
    }

    pub fn eval(
        &self,
        static_data: &HashMap<String, f64>,
        dynamic_data: &HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        for (priority, name, ast) in &self.quality_paths {
            let mut required_events = HashSet::new();
            ast.get_required_events(&mut required_events);

            if required_events.is_empty() {
                // STATIC PATH: Evaluate once with no dynamic context.
                let trace = self.evaluate_ast(ast, static_data, &HashMap::new())?;
                if let Value::Bool(true) = trace.get_outcome() {
                    return Ok(EvaluationResult {
                        quality_name: Some(name.clone()),
                        quality_priority: Some(*priority),
                        reason: self.format_trace(&trace),
                    });
                }
            } else {
                // DYNAMIC PATH: This is where we handle the cross-product of events.
                let event_list: Vec<String> = required_events.into_iter().collect();

                // This will hold the specific defects for the current evaluation context,
                // e.g., {"hole": &hole1, "tear": &tear2}
                let mut context: HashMap<String, &HashMap<String, f64>> = HashMap::new();

                if let Some(trace) = self.eval_cross_product(
                    ast,
                    &event_list,
                    &mut context,
                    static_data,
                    dynamic_data,
                )? {
                    // The recursive evaluation found a combination of defects that triggered the rule.
                    // Since we iterate by priority, this is our final answer.
                    return Ok(EvaluationResult {
                        quality_name: Some(name.clone()),
                        quality_priority: Some(*priority),
                        reason: self.format_trace(&trace),
                    });
                }
            }
        }

        Ok(EvaluationResult {
            quality_name: None,
            quality_priority: None,
            reason: "No quality triggered".to_string(),
        })
    }

    /// Recursively evaluates an AST by building a Cartesian product.
    /// Returns Some(trace) on the first combination that triggers the rule.
    fn eval_cross_product<'a>(
        &self,
        ast: &Expression,
        event_types: &[String],
        context: &mut HashMap<String, &'a HashMap<String, f64>>,
        static_data: &HashMap<String, f64>,
        dynamic_data: &'a HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Result<Option<EvaluationTrace>, EvaluationError> {
        // BASE CASE: If we have no more event types to select, our context is complete.
        // We can now evaluate the AST with this specific combination of defects.
        if event_types.is_empty() {
            let trace = self.evaluate_ast(ast, static_data, context)?;
            if let Value::Bool(true) = trace.get_outcome() {
                return Ok(Some(trace));
            }
            return Ok(None);
        }

        // RECURSIVE STEP:
        let current_event_type = &event_types[0];
        let remaining_event_types = &event_types[1..];

        // Get all defects for the current event type we need to process.
        if let Some(defects) = dynamic_data.get(current_event_type) {
            for defect in defects {
                // Add the current defect to the context.
                context.insert(current_event_type.clone(), defect);

                // Recurse to select defects for the *remaining* event types.
                if let Some(trace) = self.eval_cross_product(
                    ast,
                    remaining_event_types,
                    context,
                    static_data,
                    dynamic_data,
                )? {
                    return Ok(Some(trace));
                }
            }
        }

        // If we looped through all defects of this type and found no trigger, this path is None.
        Ok(None)
    }

    /// Formats the final trace into a human-readable string.
    fn format_trace(&self, trace: &EvaluationTrace) -> String {
        match trace {
            EvaluationTrace::BinaryOp {
                op_symbol,
                left,
                right,
                ..
            } => {
                // Recursively format the left and right sides and join with the operator
                format!(
                    "{} {} {}",
                    self.format_trace(left),
                    op_symbol,
                    self.format_trace(right)
                )
            }
            EvaluationTrace::UnaryOp {
                op_symbol, child, ..
            } => {
                // Format unary ops like "NOT (some_expression)"
                format!("{} ({})", op_symbol, self.format_trace(child))
            }
            EvaluationTrace::Leaf { source, value } => {
                if source.starts_with('$') {
                    format!("{} (was {})", source, value)
                } else {
                    source.clone()
                }
            }
            EvaluationTrace::NotEvaluated => "[Not Evaluated]".to_string(),
        }
    }

    /// Recursively evaluates an AST node against a complete context.
    fn evaluate_ast<'a>(
        &self,
        expr: &Expression,
        static_data: &HashMap<String, f64>,
        context: &HashMap<String, &'a HashMap<String, f64>>,
    ) -> Result<EvaluationTrace, EvaluationError> {
        macro_rules! eval_binary_op {
            ($l:expr, $r:expr, $op_symbol:expr, $logic:expr) => {{
                let left_trace = self.evaluate_ast($l, static_data, context)?;
                let right_trace = self.evaluate_ast($r, static_data, context)?;
                let outcome = $logic(left_trace.get_outcome(), right_trace.get_outcome())?;
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: $op_symbol,
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }};
        }

        macro_rules! eval_unary_op {
            ($v:expr, $op_symbol:expr, $logic:expr) => {{
                let child_trace = self.evaluate_ast($v, static_data, context)?;
                let outcome = $logic(child_trace.get_outcome())?;
                Ok(EvaluationTrace::UnaryOp {
                    op_symbol: $op_symbol,
                    child: Box::new(child_trace),
                    outcome,
                })
            }};
        }

        match expr {
            // Arithmetic
            Expression::Sum(l, r) => eval_binary_op!(l, r, "+", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Number(lv + rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::Subtract(l, r) => eval_binary_op!(l, r, "-", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Number(lv - rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::Multiply(l, r) => eval_binary_op!(l, r, "*", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Number(lv * rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::Divide(l, r) => eval_binary_op!(l, r, "/", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Number(lv / rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::Abs(v) => eval_unary_op!(v, "ABS", |v| match v {
                Value::Number(val) => Ok(Value::Number(val.abs())),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),

            // Logic
            Expression::Not(v) => eval_unary_op!(v, "NOT", |v| match v {
                Value::Bool(val) => Ok(Value::Bool(!val)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Bool".into(),
                    found: "Other".into()
                }),
            }),
            Expression::And(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                if let Value::Bool(false) = left_trace.get_outcome() {
                    return Ok(EvaluationTrace::BinaryOp {
                        op_symbol: "AND",
                        left: Box::new(left_trace),
                        right: Box::new(EvaluationTrace::NotEvaluated),
                        outcome: Value::Bool(false),
                    });
                }
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv && rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Bool".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "AND",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Or(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                if let Value::Bool(true) = left_trace.get_outcome() {
                    return Ok(EvaluationTrace::BinaryOp {
                        op_symbol: "OR",
                        left: Box::new(left_trace),
                        right: Box::new(EvaluationTrace::NotEvaluated),
                        outcome: Value::Bool(true),
                    });
                }
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv || rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Bool".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "OR",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Xor(l, r) => eval_binary_op!(l, r, "XOR", |l, r| match (l, r) {
                (Value::Bool(lv), Value::Bool(rv)) => Ok(Value::Bool(lv ^ rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Bool".into(),
                    found: "Other".into()
                }),
            }),

            // Comparison
            Expression::Equal(l, r) => eval_binary_op!(l, r, "==", |l, r| Ok(Value::Bool(l == r))),
            Expression::NotEqual(l, r) => {
                eval_binary_op!(l, r, "!=", |l, r| Ok(Value::Bool(l != r)))
            }
            Expression::GreaterThan(l, r) => eval_binary_op!(l, r, ">", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Bool(lv > rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::GreaterThanOrEqual(l, r) => {
                eval_binary_op!(l, r, ">=", |l, r| match (l, r) {
                    (Value::Number(lv), Value::Number(rv)) => Ok(Value::Bool(lv >= rv)),
                    _ => Err(EvaluationError::TypeMismatch {
                        expected: "Number".into(),
                        found: "Other".into()
                    }),
                })
            }
            Expression::SmallerThan(l, r) => eval_binary_op!(l, r, "<", |l, r| match (l, r) {
                (Value::Number(lv), Value::Number(rv)) => Ok(Value::Bool(lv < rv)),
                _ => Err(EvaluationError::TypeMismatch {
                    expected: "Number".into(),
                    found: "Other".into()
                }),
            }),
            Expression::SmallerThanOrEqual(l, r) => {
                eval_binary_op!(l, r, "<=", |l, r| match (l, r) {
                    (Value::Number(lv), Value::Number(rv)) => Ok(Value::Bool(lv <= rv)),
                    _ => Err(EvaluationError::TypeMismatch {
                        expected: "Number".into(),
                        found: "Other".into()
                    }),
                })
            }

            // Leaf Nodes
            Expression::Literal(val) => Ok(EvaluationTrace::Leaf {
                source: val.to_string(),
                value: val.clone(),
            }),
            Expression::Input(source) => {
                let (source_str, value) = match source {
                    InputSource::Static { name } => (
                        format!("${}", name),
                        static_data
                            .get(name)
                            .map(|v| Value::Number(*v))
                            .ok_or(EvaluationError::InputNotFound(name.clone()))?,
                    ),
                    InputSource::Dynamic { event, field } => (
                        format!("${}.{}", event, field),
                        context
                            .get(event)
                            .and_then(|data| data.get(field))
                            .map(|v| Value::Number(*v))
                            .ok_or(EvaluationError::InputNotFound(format!(
                                "{}.{}",
                                event, field
                            )))?,
                    ),
                };
                Ok(EvaluationTrace::Leaf {
                    source: source_str,
                    value,
                })
            }
        }
    }
}
