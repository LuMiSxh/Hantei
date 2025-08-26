use crate::ast::*;
use crate::error::EvaluationError;
use crate::trace::TraceFormatter;
use std::collections::{HashMap, HashSet};

/// Evaluates compiled ASTs against runtime data
pub struct Evaluator {
    /// Quality paths sorted by priority
    pub quality_paths: Vec<(i32, String, Expression)>,
}

/// Result of evaluating all quality paths
#[derive(Debug)]
pub struct EvaluationResult {
    pub quality_name: Option<String>,
    pub quality_priority: Option<i32>,
    pub reason: String,
}

impl Evaluator {
    /// Create new evaluator with compiled quality paths
    pub fn new(quality_paths: Vec<(i32, String, Expression)>) -> Self {
        Self { quality_paths }
    }

    /// Evaluate all quality paths against provided data
    pub fn eval(
        &self,
        static_data: &HashMap<String, f64>,
        dynamic_data: &HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        // Sort by priority (lowest number = highest priority)
        let mut sorted_paths = self.quality_paths.clone();
        sorted_paths.sort_by_key(|(priority, _, _)| *priority);

        for (priority, name, ast) in &sorted_paths {
            let mut required_events = HashSet::new();
            ast.get_required_events(&mut required_events);

            if required_events.is_empty() {
                // Static path - evaluate once
                let trace = self.evaluate_ast(ast, static_data, &HashMap::new())?;
                if let Value::Bool(true) = trace.get_outcome() {
                    return Ok(EvaluationResult {
                        quality_name: Some(name.clone()),
                        quality_priority: Some(*priority),
                        reason: TraceFormatter::format_trace(&trace),
                    });
                }
            } else {
                // Dynamic path - evaluate cross-product of events
                let event_list: Vec<String> = required_events.into_iter().collect();

                // Check if all required events exist in dynamic_data
                for event_type in &event_list {
                    if !dynamic_data.contains_key(event_type) {
                        return Err(EvaluationError::InputNotFound(event_type.clone()));
                    }
                }

                let mut context: HashMap<String, &HashMap<String, f64>> = HashMap::new();

                if let Some(trace) = self.eval_cross_product(
                    ast,
                    &event_list,
                    &mut context,
                    static_data,
                    dynamic_data,
                )? {
                    return Ok(EvaluationResult {
                        quality_name: Some(name.clone()),
                        quality_priority: Some(*priority),
                        reason: TraceFormatter::format_trace(&trace),
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

    /// Recursively evaluate cross-product of dynamic events
    fn eval_cross_product<'a>(
        &self,
        ast: &Expression,
        event_types: &[String],
        context: &mut HashMap<String, &'a HashMap<String, f64>>,
        static_data: &HashMap<String, f64>,
        dynamic_data: &'a HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Result<Option<EvaluationTrace>, EvaluationError> {
        // Base case: all events assigned, evaluate AST
        if event_types.is_empty() {
            let trace = self.evaluate_ast(ast, static_data, context)?;
            if let Value::Bool(true) = trace.get_outcome() {
                return Ok(Some(trace));
            }
            return Ok(None);
        }

        // Recursive case: try each defect for current event type
        let current_event_type = &event_types[0];
        let remaining_event_types = &event_types[1..];

        if let Some(defects) = dynamic_data.get(current_event_type) {
            for defect in defects {
                context.insert(current_event_type.clone(), defect);

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

        Ok(None)
    }

    /// Evaluate an AST against a complete data context
    fn evaluate_ast<'a>(
        &self,
        expr: &Expression,
        static_data: &HashMap<String, f64>,
        context: &HashMap<String, &'a HashMap<String, f64>>,
    ) -> Result<EvaluationTrace, EvaluationError> {
        match expr {
            // Arithmetic operations
            Expression::Sum(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Number(lv + rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "+",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Subtract(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Number(lv - rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "-",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Multiply(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Number(lv * rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "*",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Divide(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Number(lv / rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "/",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Abs(v) => {
                let child_trace = self.evaluate_ast(v, static_data, context)?;
                let outcome = match child_trace.get_outcome() {
                    Value::Number(val) => Value::Number(val.abs()),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::UnaryOp {
                    op_symbol: "ABS",
                    child: Box::new(child_trace),
                    outcome,
                })
            }

            // Logical operations with short-circuiting
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
            Expression::Not(v) => {
                let child_trace = self.evaluate_ast(v, static_data, context)?;
                let outcome = match child_trace.get_outcome() {
                    Value::Bool(val) => Value::Bool(!val),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Bool".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::UnaryOp {
                    op_symbol: "NOT",
                    child: Box::new(child_trace),
                    outcome,
                })
            }
            Expression::Xor(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv ^ rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Bool".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "XOR",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }

            // Comparison operations
            Expression::Equal(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = Value::Bool(left_trace.get_outcome() == right_trace.get_outcome());
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "==",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::NotEqual(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = Value::Bool(left_trace.get_outcome() != right_trace.get_outcome());
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "!=",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::GreaterThan(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Bool(lv > rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: ">",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::GreaterThanOrEqual(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Bool(lv >= rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: ">=",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::SmallerThan(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Bool(lv < rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "<",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::SmallerThanOrEqual(l, r) => {
                let left_trace = self.evaluate_ast(l, static_data, context)?;
                let right_trace = self.evaluate_ast(r, static_data, context)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Number(lv), Value::Number(rv)) => Value::Bool(lv <= rv),
                    _ => {
                        return Err(EvaluationError::TypeMismatch {
                            expected: "Number".into(),
                            found: "Other".into(),
                        });
                    }
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "<=",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }

            // Leaf nodes
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
