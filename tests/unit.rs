//! Unit tests for core Hantei functionality.
mod common;
use hantei::error::{AstBuildError, EvaluationError, VmError};
use hantei::prelude::*;
use std::collections::HashSet;

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Number(42.0)), "42");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::Null), "null");
}

#[test]
fn test_input_source_display() {
    let static_src = InputSource::Static {
        name: "Temp".to_string(),
    };
    let dynamic_src = InputSource::Dynamic {
        event: "hole".to_string(),
        field: "Diameter".to_string(),
    };
    assert_eq!(format!("{}", static_src), "$Temp");
    assert_eq!(format!("{}", dynamic_src), "$hole.Diameter");
}

#[test]
fn test_expression_required_events() {
    let expr = Expression::And(
        Box::new(Expression::Input(InputSource::Static {
            name: "Temp".to_string(),
        })),
        Box::new(Expression::Input(InputSource::Dynamic {
            event: "hole".to_string(),
            field: "Diameter".to_string(),
        })),
    );

    let mut events = HashSet::new();
    expr.get_required_events(&mut events);
    assert_eq!(events.len(), 1);
    assert!(events.contains("hole"));
}

#[test]
fn test_trace_formatter_short_circuit() {
    let trace = EvaluationTrace::BinaryOp {
        op_symbol: "OR",
        left: Box::new(EvaluationTrace::Leaf {
            source: "true".to_string(),
            value: Value::Bool(true),
        }),
        right: Box::new(EvaluationTrace::NotEvaluated),
        outcome: Value::Bool(true),
    };

    let formatted = TraceFormatter::format_trace(&trace);
    assert_eq!(formatted, "true"); // Should only show the decisive part
}

#[test]
fn test_error_display() {
    let err = AstBuildError::NodeNotFound {
        missing_node_id: "node_B".to_string(),
        source_node_id: "node_A".to_string(),
    };
    assert!(err.to_string().contains("node_B"));
    assert!(err.to_string().contains("node_A"));

    let eval_err = EvaluationError::TypeMismatch {
        operation: "+".to_string(),
        expected: "Number".to_string(),
        found: Value::Bool(false),
    };
    assert!(eval_err.to_string().contains('+'));
    assert!(eval_err.to_string().contains("Number"));
    assert!(eval_err.to_string().contains("false"));

    let vm_err = VmError::StackUnderflow;
    assert!(vm_err.to_string().contains("Stack underflow"));
}
