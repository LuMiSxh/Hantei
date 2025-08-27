use crate::ast::{EvaluationTrace, Expression, InputSource, Value};
use crate::error::EvaluationError;
use std::collections::HashMap;

// This macro generates a match arm for a binary operation.
macro_rules! eval_op {
    ($self:ident, $l:ident, $r:ident, $op_str:expr, $op_fn:expr, number) => {
        $self.eval_binary($l, $r, $op_str, $op_fn)
    };
    ($self:ident, $l:ident, $r:ident, $op_str:expr, $op_fn:expr, bool) => {
        $self.eval_comparison($l, $r, $op_str, $op_fn)
    };
}

/// The core recursive engine for evaluating a single, fully-contextualized AST.
pub(super) struct AstEngine<'a> {
    expression: &'a Expression,
    static_data: &'a HashMap<String, f64>,
    dynamic_context: &'a HashMap<String, &'a HashMap<String, f64>>,
}

impl<'a> AstEngine<'a> {
    pub(super) fn new(
        expression: &'a Expression,
        static_data: &'a HashMap<String, f64>,
        dynamic_context: &'a HashMap<String, &'a HashMap<String, f64>>,
    ) -> Self {
        Self {
            expression,
            static_data,
            dynamic_context,
        }
    }

    /// Evaluates the AST and returns a trace of the execution.
    pub(super) fn evaluate(&self) -> Result<EvaluationTrace, EvaluationError> {
        self.evaluate_recursive(self.expression)
    }

    fn evaluate_recursive(&self, expr: &Expression) -> Result<EvaluationTrace, EvaluationError> {
        match expr {
            // --- Arithmetic Operations ---
            Expression::Sum(l, r) => eval_op!(self, l, r, "+", |a, b| a + b, number),
            Expression::Subtract(l, r) => eval_op!(self, l, r, "-", |a, b| a - b, number),
            Expression::Multiply(l, r) => eval_op!(self, l, r, "*", |a, b| a * b, number),
            Expression::Divide(l, r) => eval_op!(self, l, r, "/", |a, b| a / b, number),
            Expression::Abs(v) => {
                let child_trace = self.evaluate_recursive(v)?;
                let outcome = match child_trace.get_outcome() {
                    Value::Number(val) => Value::Number(val.abs()),
                    val => return Err(self.type_mismatch("ABS", "Number", val)),
                };
                Ok(EvaluationTrace::UnaryOp {
                    op_symbol: "ABS",
                    child: Box::new(child_trace),
                    outcome,
                })
            }

            // --- Comparison Operations ---
            Expression::GreaterThan(l, r) => eval_op!(self, l, r, ">", |a, b| a > b, bool),
            Expression::SmallerThan(l, r) => eval_op!(self, l, r, "<", |a, b| a < b, bool),
            Expression::GreaterThanOrEqual(l, r) => eval_op!(self, l, r, ">=", |a, b| a >= b, bool),
            Expression::SmallerThanOrEqual(l, r) => eval_op!(self, l, r, "<=", |a, b| a <= b, bool),

            // --- Equality ---
            Expression::Equal(l, r) => {
                let left_trace = self.evaluate_recursive(l)?;
                let right_trace = self.evaluate_recursive(r)?;
                let outcome = Value::Bool(left_trace.get_outcome() == right_trace.get_outcome());
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "==",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::NotEqual(l, r) => {
                let left_trace = self.evaluate_recursive(l)?;
                let right_trace = self.evaluate_recursive(r)?;
                let outcome = Value::Bool(left_trace.get_outcome() != right_trace.get_outcome());
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "!=",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }

            // --- Logical Operations  ---
            Expression::And(l, r) => {
                let left_trace = self.evaluate_recursive(l)?;
                if let Value::Bool(false) = left_trace.get_outcome() {
                    return Ok(EvaluationTrace::BinaryOp {
                        op_symbol: "AND",
                        left: Box::new(left_trace),
                        right: Box::new(EvaluationTrace::NotEvaluated),
                        outcome: Value::Bool(false),
                    });
                }
                let right_trace = self.evaluate_recursive(r)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv && rv),
                    (l_val, _) => return Err(self.type_mismatch("AND", "Bool", l_val)),
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "AND",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Or(l, r) => {
                let left_trace = self.evaluate_recursive(l)?;
                if let Value::Bool(true) = left_trace.get_outcome() {
                    return Ok(EvaluationTrace::BinaryOp {
                        op_symbol: "OR",
                        left: Box::new(left_trace),
                        right: Box::new(EvaluationTrace::NotEvaluated),
                        outcome: Value::Bool(true),
                    });
                }
                let right_trace = self.evaluate_recursive(r)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv || rv),
                    (l_val, _) => return Err(self.type_mismatch("OR", "Bool", l_val)),
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "OR",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }
            Expression::Not(v) => {
                let child_trace = self.evaluate_recursive(v)?;
                let outcome = match child_trace.get_outcome() {
                    Value::Bool(val) => Value::Bool(!val),
                    val => return Err(self.type_mismatch("NOT", "Bool", val)),
                };
                Ok(EvaluationTrace::UnaryOp {
                    op_symbol: "NOT",
                    child: Box::new(child_trace),
                    outcome,
                })
            }
            Expression::Xor(l, r) => {
                let left_trace = self.evaluate_recursive(l)?;
                let right_trace = self.evaluate_recursive(r)?;
                let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
                    (Value::Bool(lv), Value::Bool(rv)) => Value::Bool(lv ^ rv),
                    (l_val, _) => return Err(self.type_mismatch("XOR", "Bool", l_val)),
                };
                Ok(EvaluationTrace::BinaryOp {
                    op_symbol: "XOR",
                    left: Box::new(left_trace),
                    right: Box::new(right_trace),
                    outcome,
                })
            }

            // --- Other Operations ---
            Expression::Literal(val) => Ok(EvaluationTrace::Leaf {
                source: val.to_string(),
                value: val.clone(),
            }),
            Expression::Input(source) => {
                let (source_str, value) = match source {
                    InputSource::Static { name } => (
                        format!("${}", name),
                        self.static_data
                            .get(name)
                            .map(|v| Value::Number(*v))
                            .ok_or_else(|| EvaluationError::InputNotFound(name.clone()))?,
                    ),
                    InputSource::Dynamic { event, field } => (
                        format!("${}.{}", event, field),
                        self.dynamic_context
                            .get(event)
                            .and_then(|data| data.get(field))
                            .map(|v| Value::Number(*v))
                            .ok_or_else(|| {
                                EvaluationError::InputNotFound(format!("{}.{}", event, field))
                            })?,
                    ),
                };
                Ok(EvaluationTrace::Leaf {
                    source: source_str,
                    value,
                })
            }
        }
    }

    fn eval_binary<F>(
        &self,
        l: &Expression,
        r: &Expression,
        op: &'static str,
        f: F,
    ) -> Result<EvaluationTrace, EvaluationError>
    where
        F: Fn(f64, f64) -> f64,
    {
        let left_trace = self.evaluate_recursive(l)?;
        let right_trace = self.evaluate_recursive(r)?;
        let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
            (Value::Number(lv), Value::Number(rv)) => Value::Number(f(lv, rv)),
            (l_val, _) => return Err(self.type_mismatch(op, "Number", l_val)),
        };
        Ok(EvaluationTrace::BinaryOp {
            op_symbol: op,
            left: Box::new(left_trace),
            right: Box::new(right_trace),
            outcome,
        })
    }

    fn eval_comparison<F>(
        &self,
        l: &Expression,
        r: &Expression,
        op: &'static str,
        f: F,
    ) -> Result<EvaluationTrace, EvaluationError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_trace = self.evaluate_recursive(l)?;
        let right_trace = self.evaluate_recursive(r)?;
        let outcome = match (left_trace.get_outcome(), right_trace.get_outcome()) {
            (Value::Number(lv), Value::Number(rv)) => Value::Bool(f(lv, rv)),
            (l_val, _) => return Err(self.type_mismatch(op, "Number", l_val)),
        };
        Ok(EvaluationTrace::BinaryOp {
            op_symbol: op,
            left: Box::new(left_trace),
            right: Box::new(right_trace),
            outcome,
        })
    }

    fn type_mismatch(&self, op: &str, expected: &str, found: Value) -> EvaluationError {
        EvaluationError::TypeMismatch {
            operation: op.to_string(),
            expected: expected.to_string(),
            found,
        }
    }
}
