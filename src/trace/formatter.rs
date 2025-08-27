use crate::ast::{EvaluationTrace, Value};
use std::fmt::Write;

/// Formats evaluation traces into human-readable strings.
pub struct TraceFormatter;

impl TraceFormatter {
    /// Formats the decisive parts of a trace into a concise explanation.
    pub fn format_trace(trace: &EvaluationTrace) -> String {
        let mut reasons = Vec::new();
        Self::collect_decisive_reasons(trace, &mut reasons);

        if reasons.is_empty() {
            Self::format_full_expression(trace)
        } else {
            reasons.join(" AND ")
        }
    }

    /// Recursively collects only the parts of the trace that were decisive for the outcome.
    fn collect_decisive_reasons(trace: &EvaluationTrace, reasons: &mut Vec<String>) {
        match trace {
            EvaluationTrace::BinaryOp {
                op_symbol,
                left,
                right,
                outcome,
            } => match (*op_symbol, outcome.clone()) {
                ("AND", Value::Bool(true)) => {
                    Self::collect_decisive_reasons(left, reasons);
                    Self::collect_decisive_reasons(right, reasons);
                }
                ("AND", Value::Bool(false)) => {
                    if let Value::Bool(false) = left.get_outcome() {
                        Self::collect_decisive_reasons(left, reasons);
                    } else {
                        Self::collect_decisive_reasons(right, reasons);
                    }
                }
                ("OR", Value::Bool(true)) => {
                    if let Value::Bool(true) = left.get_outcome() {
                        Self::collect_decisive_reasons(left, reasons);
                    } else {
                        Self::collect_decisive_reasons(right, reasons);
                    }
                }
                ("OR", Value::Bool(false)) => {
                    Self::collect_decisive_reasons(left, reasons);
                    Self::collect_decisive_reasons(right, reasons);
                }
                _ => reasons.push(Self::format_full_expression(trace)),
            },
            _ => {
                let formatted = Self::format_full_expression(trace);
                if !formatted.is_empty() {
                    reasons.push(formatted);
                }
            }
        }
    }

    fn format_full_expression(trace: &EvaluationTrace) -> String {
        Self::format_recursive(trace, 0)
    }

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
                if source.starts_with('$') {
                    write!(result, "{} (was {})", source, Self::format_value(value)).unwrap();
                } else {
                    result.push_str(source);
                }
            }
            EvaluationTrace::NotEvaluated => {}
        }

        if needs_parens {
            result.push(')');
        }
        result
    }

    fn format_value(value: &Value) -> String {
        value.to_string()
    }
}
