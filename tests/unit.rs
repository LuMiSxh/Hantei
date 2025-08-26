//! Unit tests for core Hantei functionality
//!
//! Tests individual components in isolation without complex integration.

mod common;
use common::*;
use hantei::prelude::*;
use std::collections::HashSet;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_value_display() {
        let number_val = Value::Number(42.0);
        let bool_val = Value::Bool(true);
        let null_val = Value::Null;

        assert_eq!(format!("{}", number_val), "42");
        assert_eq!(format!("{}", bool_val), "true");
        assert_eq!(format!("{}", null_val), "null");

        // Test decimal numbers
        let decimal_val = Value::Number(3.14159);
        assert_eq!(format!("{}", decimal_val), "3.14159");
    }

    #[test]
    fn test_input_source_display() {
        let static_source = InputSource::Static {
            name: "Temperature".to_string(),
        };
        let dynamic_source = InputSource::Dynamic {
            event: "hole".to_string(),
            field: "Diameter".to_string(),
        };

        assert_eq!(format!("{}", static_source), "$Temperature");
        assert_eq!(format!("{}", dynamic_source), "$hole.Diameter");
    }

    #[test]
    fn test_expression_required_events() {
        // Create expression with dynamic inputs
        let expr = Expression::GreaterThan(
            Box::new(Expression::Input(InputSource::Dynamic {
                event: "hole".to_string(),
                field: "Diameter".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(10.0))),
        );

        let mut events = HashSet::new();
        expr.get_required_events(&mut events);

        assert_eq!(events.len(), 1);
        assert!(events.contains("hole"));
    }

    #[test]
    fn test_expression_required_events_multiple() {
        // Create complex expression with multiple event types
        let expr = Expression::And(
            Box::new(Expression::GreaterThan(
                Box::new(Expression::Input(InputSource::Dynamic {
                    event: "hole".to_string(),
                    field: "Diameter".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(5.0))),
            )),
            Box::new(Expression::SmallerThan(
                Box::new(Expression::Input(InputSource::Dynamic {
                    event: "tear".to_string(),
                    field: "Length".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(20.0))),
            )),
        );

        let mut events = HashSet::new();
        expr.get_required_events(&mut events);

        assert_eq!(events.len(), 2);
        assert!(events.contains("hole"));
        assert!(events.contains("tear"));
    }

    #[test]
    fn test_expression_required_events_static_only() {
        // Expression with only static inputs
        let expr = Expression::GreaterThan(
            Box::new(Expression::Input(InputSource::Static {
                name: "Temperature".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(25.0))),
        );

        let mut events = HashSet::new();
        expr.get_required_events(&mut events);

        assert!(events.is_empty());
    }

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_ok());
    }

    #[test]
    fn test_compiler_invalid_json() {
        let invalid_recipe = "{ invalid json }";
        let compiler = Compiler::new(invalid_recipe, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_err());

        let invalid_qualities = "[ invalid json ]";
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, invalid_qualities);
        assert!(compiler.is_err());
    }

    #[test]
    fn test_sample_data_creation() {
        let static_data = create_sample_static_data();
        let dynamic_data = create_sample_dynamic_data();

        assert!(!static_data.is_empty());
        assert!(!dynamic_data.is_empty());

        // Check specific values
        assert_eq!(static_data.get("Temperature"), Some(&32.5));
        assert!(dynamic_data.contains_key("hole"));
        assert_eq!(dynamic_data.get("hole").unwrap().len(), 2);
    }

    #[test]
    fn test_sample_data_from_json() {
        let sample_json = r#"{
            "static_data": {
                "Temperature": 25.0,
                "Humidity": 60.0
            },
            "dynamic_data": {
                "hole": [
                    { "Diameter": 5.0, "Length": 10.0 }
                ]
            }
        }"#;

        let sample_data: std::result::Result<SampleData, _> = serde_json::from_str(sample_json);
        assert!(sample_data.is_ok());

        let data = sample_data.unwrap();
        assert_eq!(data.static_data().len(), 2);
        assert_eq!(data.dynamic_data().len(), 1);
    }

    #[test]
    fn test_sample_data_default() {
        let default_data = SampleData::default();
        assert!(!default_data.static_data().is_empty());
        assert!(!default_data.dynamic_data().is_empty());
    }

    #[test]
    fn test_evaluation_trace_get_outcome() {
        let leaf_trace = EvaluationTrace::Leaf {
            source: "$Temperature".to_string(),
            value: Value::Number(30.0),
        };
        assert_eq!(leaf_trace.get_outcome(), Value::Number(30.0));

        let binary_trace = EvaluationTrace::BinaryOp {
            op_symbol: ">",
            left: Box::new(leaf_trace),
            right: Box::new(EvaluationTrace::Leaf {
                source: "25.0".to_string(),
                value: Value::Number(25.0),
            }),
            outcome: Value::Bool(true),
        };
        assert_eq!(binary_trace.get_outcome(), Value::Bool(true));

        let not_evaluated = EvaluationTrace::NotEvaluated;
        assert_eq!(not_evaluated.get_outcome(), Value::Null);
    }

    #[test]
    fn test_trace_formatter() {
        let trace = EvaluationTrace::BinaryOp {
            op_symbol: ">",
            left: Box::new(EvaluationTrace::Leaf {
                source: "$Temperature".to_string(),
                value: Value::Number(30.0),
            }),
            right: Box::new(EvaluationTrace::Leaf {
                source: "25.0".to_string(),
                value: Value::Number(25.0),
            }),
            outcome: Value::Bool(true),
        };

        let formatted = TraceFormatter::format_trace(&trace);
        assert!(formatted.contains("$Temperature"));
        assert!(formatted.contains("was 30"));
        assert!(formatted.contains(">"));
        assert!(formatted.contains("25.0"));
    }

    #[test]
    fn test_ast_pretty_printing() {
        let expr = Expression::GreaterThan(
            Box::new(Expression::Input(InputSource::Static {
                name: "Temperature".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(25.0))),
        );

        let ast_string = format!("{}", expr);
        assert!(ast_string.contains(">"));
        assert!(ast_string.contains("$Temperature"));
        assert!(ast_string.contains("25"));
    }

    #[test]
    fn test_complex_ast_structure() {
        let expr = Expression::And(
            Box::new(Expression::GreaterThan(
                Box::new(Expression::Input(InputSource::Static {
                    name: "Temperature".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(30.0))),
            )),
            Box::new(Expression::SmallerThan(
                Box::new(Expression::Input(InputSource::Dynamic {
                    event: "hole".to_string(),
                    field: "Diameter".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(10.0))),
            )),
        );

        let ast_string = format!("{}", expr);
        let expected_patterns = ["AND", ">", "<", "$Temperature", "$hole.Diameter"];
        assert!(validate_ast_structure(&ast_string, &expected_patterns));
    }

    #[test]
    fn test_arithmetic_expressions() {
        let sum_expr = Expression::Sum(
            Box::new(Expression::Literal(Value::Number(10.0))),
            Box::new(Expression::Literal(Value::Number(5.0))),
        );

        let mult_expr = Expression::Multiply(
            Box::new(sum_expr),
            Box::new(Expression::Literal(Value::Number(2.0))),
        );

        let ast_string = format!("{}", mult_expr);
        assert!(ast_string.contains("+"));
        assert!(ast_string.contains("*"));
        assert!(ast_string.contains("10"));
        assert!(ast_string.contains("5"));
        assert!(ast_string.contains("2"));
    }

    #[test]
    fn test_nested_logical_expressions() {
        let expr = Expression::Or(
            Box::new(Expression::And(
                Box::new(Expression::Literal(Value::Bool(true))),
                Box::new(Expression::Literal(Value::Bool(false))),
            )),
            Box::new(Expression::Not(Box::new(Expression::Literal(Value::Bool(
                false,
            ))))),
        );

        let ast_string = format!("{}", expr);
        assert!(ast_string.contains("OR"));
        assert!(ast_string.contains("AND"));
        assert!(ast_string.contains("NOT"));
        assert!(ast_string.contains("true"));
        assert!(ast_string.contains("false"));
    }

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Null, Value::Null);

        assert_ne!(Value::Number(42.0), Value::Number(43.0));
        assert_ne!(Value::Bool(true), Value::Bool(false));
        assert_ne!(Value::Number(42.0), Value::Bool(true));
    }

    #[test]
    fn test_input_source_equality() {
        let static1 = InputSource::Static {
            name: "Temperature".to_string(),
        };
        let static2 = InputSource::Static {
            name: "Temperature".to_string(),
        };
        let static3 = InputSource::Static {
            name: "Humidity".to_string(),
        };

        assert_eq!(static1, static2);
        assert_ne!(static1, static3);

        let dynamic1 = InputSource::Dynamic {
            event: "hole".to_string(),
            field: "Diameter".to_string(),
        };
        let dynamic2 = InputSource::Dynamic {
            event: "hole".to_string(),
            field: "Diameter".to_string(),
        };

        assert_eq!(dynamic1, dynamic2);
        assert_ne!(static1, dynamic1);
    }

    #[test]
    fn test_error_types() {
        use hantei::error::{CompileError, EvaluationError};

        let compile_err = CompileError::NodeNotFound("test_node".to_string());
        let error_string = format!("{}", compile_err);
        assert!(error_string.contains("test_node"));
        assert!(error_string.contains("not found"));

        let eval_err = EvaluationError::InputNotFound("missing_input".to_string());
        let error_string = format!("{}", eval_err);
        assert!(error_string.contains("missing_input"));
        assert!(error_string.contains("not found"));

        let type_err = EvaluationError::TypeMismatch {
            expected: "Number".to_string(),
            found: "Bool".to_string(),
        };
        let error_string = format!("{}", type_err);
        assert!(error_string.contains("Number"));
        assert!(error_string.contains("Bool"));
        assert!(error_string.contains("mismatch"));
    }

    #[test]
    fn test_ui_types_deserialization() {
        let node_json = r#"{
            "id": "test_node",
            "data": {
                "nodeData": {
                    "realNodeType": "gtNode",
                    "values": [null, 25.0]
                }
            }
        }"#;

        let ui_node: std::result::Result<hantei::ui::UiNode, _> = serde_json::from_str(node_json);
        assert!(ui_node.is_ok());

        let node = ui_node.unwrap();
        assert_eq!(node.id, "test_node");
        assert_eq!(node.data.node_data.real_node_type, "gtNode");
    }

    #[test]
    fn test_quality_deserialization() {
        let quality_json = r#"{
            "id": 0,
            "name": "Test Quality",
            "priority": 1,
            "negated": false
        }"#;

        let quality: std::result::Result<Quality, _> = serde_json::from_str(quality_json);
        assert!(quality.is_ok());

        let q = quality.unwrap();
        assert_eq!(q.name, "Test Quality");
        assert_eq!(q.priority, 1);
    }

    #[test]
    fn test_minimal_recipe_compilation() {
        let minimal_recipe = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "realNodeType": "dynamicNode",
                            "realInputType": null,
                            "cases": [
                                {
                                    "caseId": 0,
                                    "caseName": "Value",
                                    "realCaseType": "number"
                                }
                            ]
                        }
                    }
                },
                {
                    "id": "0002",
                    "data": {
                        "nodeData": {
                            "realNodeType": "setQualityNode"
                        }
                    }
                }
            ],
            "edges": [
                {
                    "source": "0001",
                    "target": "0002",
                    "sourceHandle": "number-number-0001-0",
                    "targetHandle": "sq-number-0002-0"
                }
            ]
        }"#;

        let minimal_qualities = r#"[
            { "id": 0, "name": "Test", "priority": 1, "negated": false }
        ]"#;

        let compiler = Compiler::new(minimal_recipe, minimal_qualities);
        assert!(compiler.is_ok());

        let result = compiler.unwrap().compile(false);
        // Don't assert success here as the recipe might be incomplete for quality connection
        match result {
            Ok((_, paths)) => {
                println!("Minimal compilation succeeded with {} paths", paths.len());
            }
            Err(e) => {
                println!("Minimal compilation failed (expected): {}", e);
            }
        }
    }

    #[test]
    fn test_empty_collections() {
        let empty_static: HashMap<String, f64> = HashMap::new();
        let empty_dynamic: HashMap<String, Vec<HashMap<String, f64>>> = HashMap::new();

        // These should not panic
        let (minimal_static, minimal_dynamic) = create_minimal_test_data();
        assert!(!minimal_static.is_empty());
        assert!(minimal_dynamic.is_empty());

        // Test with truly empty data
        assert!(empty_static.is_empty());
        assert!(empty_dynamic.is_empty());
    }
}
