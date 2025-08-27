use crate::ast::{EvaluationTrace, Value};

/// Formats evaluation traces into human-readable strings
pub struct TraceFormatter;

impl TraceFormatter {
    /// Format an evaluation trace into a human-readable explanation.
    pub fn format_trace(trace: &EvaluationTrace) -> String {
        // Start the recursive formatting with the lowest possible parent precedence.
        Self::format_recursive(trace, 0)
    }

    /// Recursively formats the trace, adding parentheses only when necessary.
    fn format_recursive(trace: &EvaluationTrace, parent_precedence: u8) -> String {
        let current_precedence = trace.precedence();
        let needs_parens = current_precedence < parent_precedence;

        let mut result = String::new();
        if needs_parens {
            result.push('(');
        }

        match trace {
            EvaluationTrace::BinaryOp {
                op_symbol,
                left,
                right,
                ..
            } => {
                let left_str = Self::format_recursive(left, current_precedence);

                // For short-circuiting operators, only include the right side if it was evaluated.
                if !matches!(**right, EvaluationTrace::NotEvaluated) {
                    let right_str = Self::format_recursive(right, current_precedence);
                    result.push_str(&format!("{} {} {}", left_str, op_symbol, right_str));
                } else {
                    // If short-circuited, just show the left side that caused the result.
                    result.push_str(&left_str);
                }
            }
            EvaluationTrace::UnaryOp {
                op_symbol, child, ..
            } => {
                let child_str = Self::format_recursive(child, current_precedence);
                result.push_str(&format!("{} {}", op_symbol, child_str));
            }
            EvaluationTrace::Leaf { source, value } => {
                let formatted_leaf = if source.starts_with('$') {
                    format!("{} (was {})", source, Self::format_value(value))
                } else {
                    source.clone()
                };
                result.push_str(&formatted_leaf);
            }
            // This case is now only hit if a branch is explicitly NotEvaluated, and will be skipped by the BinaryOp logic.
            EvaluationTrace::NotEvaluated => {}
        }

        if needs_parens {
            result.push(')');
        }
        result
    }

    /// Format a value for display.
    fn format_value(value: &Value) -> String {
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => format!("{}", b),
            Value::Null => "null".to_string(),
        }
    }
}
