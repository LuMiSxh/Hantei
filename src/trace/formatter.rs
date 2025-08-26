use crate::ast::{EvaluationTrace, Value};

/// Formats evaluation traces into human-readable strings
pub struct TraceFormatter;

impl TraceFormatter {
    /// Format an evaluation trace into a human-readable explanation
    pub fn format_trace(trace: &EvaluationTrace) -> String {
        Self::format_trace_recursive(trace)
    }

    /// Recursively format trace components
    fn format_trace_recursive(trace: &EvaluationTrace) -> String {
        match trace {
            EvaluationTrace::BinaryOp {
                op_symbol,
                left,
                right,
                ..
            } => {
                // Format binary operations like "A > B" or "X AND Y"
                format!(
                    "{} {} {}",
                    Self::format_trace_recursive(left),
                    op_symbol,
                    Self::format_trace_recursive(right)
                )
            }
            EvaluationTrace::UnaryOp {
                op_symbol, child, ..
            } => {
                // Format unary operations like "NOT (expression)"
                format!("{} ({})", op_symbol, Self::format_trace_recursive(child))
            }
            EvaluationTrace::Leaf { source, value } => {
                // Format leaf nodes with their source and evaluated value
                if source.starts_with('$') {
                    format!("{} (was {})", source, Self::format_value(value))
                } else {
                    source.clone()
                }
            }
            EvaluationTrace::NotEvaluated => "[Not Evaluated]".to_string(),
        }
    }

    /// Format a value for display
    fn format_value(value: &Value) -> String {
        match value {
            Value::Number(n) => {
                // Format numbers nicely, removing unnecessary decimals
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
