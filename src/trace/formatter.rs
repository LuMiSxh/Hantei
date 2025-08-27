use crate::ast::{EvaluationTrace, Value};
use std::fmt::Write;

/// Formats evaluation traces into human-readable strings, focusing on
/// the decisive parts of the logic that led to the final outcome.
pub struct TraceFormatter;

impl TraceFormatter {
    /// Formats the decisive parts of an evaluation trace into a concise,
    /// human-readable explanation.
    pub fn format_trace(trace: &EvaluationTrace) -> String {
        let mut reasons = Vec::new();
        Self::collect_decisive_reasons(trace, &mut reasons);

        if reasons.is_empty() {
            // Fallback for simple cases like a single literal value
            Self::format_full_expression(trace)
        } else {
            reasons.join(" AND ")
        }
    }

    /// Recursively collects only the parts of the trace that were
    /// necessary for the final outcome.
    fn collect_decisive_reasons(trace: &EvaluationTrace, reasons: &mut Vec<String>) {
        match trace {
            EvaluationTrace::BinaryOp {
                op_symbol,
                left,
                right,
                outcome,
            } => {
                match (*op_symbol, outcome.clone()) {
                    // AND is true: Both sides were decisive.
                    ("AND", Value::Bool(true)) => {
                        Self::collect_decisive_reasons(left, reasons);
                        Self::collect_decisive_reasons(right, reasons);
                    }
                    // AND is false: The first side that was false is the only reason.
                    ("AND", Value::Bool(false)) => {
                        if let Value::Bool(false) = left.get_outcome() {
                            Self::collect_decisive_reasons(left, reasons);
                        } else {
                            Self::collect_decisive_reasons(right, reasons);
                        }
                    }
                    // OR is true: The first side that was true is the only reason.
                    ("OR", Value::Bool(true)) => {
                        if let Value::Bool(true) = left.get_outcome() {
                            Self::collect_decisive_reasons(left, reasons);
                        } else {
                            Self::collect_decisive_reasons(right, reasons);
                        }
                    }
                    // OR is false: Both sides were decisive.
                    ("OR", Value::Bool(false)) => {
                        Self::collect_decisive_reasons(left, reasons);
                        Self::collect_decisive_reasons(right, reasons);
                    }
                    // For any other operation (>, <, +, ==, etc.), the entire
                    // expression is considered a single, decisive unit.
                    _ => {
                        reasons.push(Self::format_full_expression(trace));
                    }
                }
            }
            // For leaf nodes or unary operations, the expression itself is the reason.
            _ => {
                let formatted = Self::format_full_expression(trace);
                if !formatted.is_empty() {
                    reasons.push(formatted);
                }
            }
        }
    }

    /// Formats a single expression trace without pruning, used as a building
    /// block for the decisive reason string.
    fn format_full_expression(trace: &EvaluationTrace) -> String {
        Self::format_recursive(trace, 0)
    }

    /// Recursively formats the trace, adding parentheses only when necessary.
    /// (This function is mostly unchanged from your original).
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
                if !matches!(**right, EvaluationTrace::NotEvaluated) {
                    let right_str = Self::format_recursive(right, current_precedence);
                    write!(result, "{} {} {}", left_str, op_symbol, right_str).unwrap();
                } else {
                    result.push_str(&left_str);
                }
            }
            EvaluationTrace::UnaryOp {
                op_symbol, child, ..
            } => {
                let child_str = Self::format_recursive(child, current_precedence);
                write!(result, "{} {}", op_symbol, child_str).unwrap();
            }
            EvaluationTrace::Leaf { source, value } => {
                let formatted_leaf = if source.starts_with('$') {
                    format!("{} (was {})", source, Self::format_value(value))
                } else {
                    source.clone()
                };
                result.push_str(&formatted_leaf);
            }
            EvaluationTrace::NotEvaluated => {}
        }

        if needs_parens {
            result.push(')');
        }
        result
    }

    /// Format a value for display. (Unchanged).
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
