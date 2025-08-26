//! Evaluator-specific functionality tests
//!
//! Tests for the evaluation engine, trace generation, and runtime data handling.

use hantei::prelude::*;
use std::collections::HashMap;
mod common;
use common::*;

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let compiled_paths = vec![(
            1,
            "Test Quality".to_string(),
            Expression::Literal(Value::Bool(true)),
        )];

        let evaluator = Evaluator::new(compiled_paths);
        assert_eq!(evaluator.quality_paths.len(), 1);
        assert_eq!(evaluator.quality_paths[0].0, 1);
        assert_eq!(evaluator.quality_paths[0].1, "Test Quality");
    }

    #[test]
    fn test_evaluator_empty_paths() {
        let evaluator = Evaluator::new(vec![]);

        let static_data = HashMap::new();
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Empty evaluator should not fail");

        assert!(result.quality_name.is_none());
        assert!(result.quality_priority.is_none());
        assert_eq!(result.reason, "No quality triggered");
    }

    #[test]
    fn test_evaluator_static_path_evaluation() {
        // Create a simple static expression: true
        let compiled_paths = vec![
            (
                1,
                "Always True".to_string(),
                Expression::Literal(Value::Bool(true)),
            ),
            (
                2,
                "Always False".to_string(),
                Expression::Literal(Value::Bool(false)),
            ),
        ];

        let evaluator = Evaluator::new(compiled_paths);
        let static_data = HashMap::new();
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Static evaluation should succeed");

        assert_eq!(result.quality_name, Some("Always True".to_string()));
        assert_eq!(result.quality_priority, Some(1));
        assert!(!result.reason.is_empty());
        println!("Static evaluation result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_static_input_resolution() {
        // Create expression that uses static input: $Temperature > 25
        let expr = Expression::GreaterThan(
            Box::new(Expression::Input(InputSource::Static {
                name: "Temperature".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(25.0))),
        );

        let compiled_paths = vec![(1, "Hot".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        // Test with high temperature
        let mut static_data = HashMap::new();
        static_data.insert("Temperature".to_string(), 30.0);
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Static input evaluation should succeed");

        assert_eq!(result.quality_name, Some("Hot".to_string()));
        assert!(result.reason.contains("Temperature"));
        assert!(result.reason.contains("30"));
        println!("Hot temperature result: {}", result.reason);

        // Test with low temperature
        static_data.insert("Temperature".to_string(), 20.0);

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Static input evaluation should succeed");

        assert!(result.quality_name.is_none());
        println!("Cold temperature result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_missing_static_input() {
        let expr = Expression::Input(InputSource::Static {
            name: "MissingValue".to_string(),
        });

        let compiled_paths = vec![(1, "Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);
        let static_data = HashMap::new(); // Missing the required input
        let dynamic_data = HashMap::new();

        let result = evaluator.eval(&static_data, &dynamic_data);
        assert!(result.is_err());

        match result {
            Err(EvaluationError::InputNotFound(name)) => {
                assert_eq!(name, "MissingValue");
                println!("Correctly handled missing static input: {}", name);
            }
            _ => panic!("Expected InputNotFound error"),
        }
    }

    #[test]
    fn test_evaluator_dynamic_input_resolution() {
        // Create expression that uses dynamic input: $hole.Diameter < 10
        let expr = Expression::SmallerThan(
            Box::new(Expression::Input(InputSource::Dynamic {
                event: "hole".to_string(),
                field: "Diameter".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(10.0))),
        );

        let compiled_paths = vec![(1, "Small Hole".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let mut dynamic_data = HashMap::new();

        // Create hole event with small diameter
        let mut hole_event = HashMap::new();
        hole_event.insert("Diameter".to_string(), 8.0);
        dynamic_data.insert("hole".to_string(), vec![hole_event]);

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Dynamic input evaluation should succeed");

        assert_eq!(result.quality_name, Some("Small Hole".to_string()));
        assert!(result.reason.contains("hole.Diameter"));
        assert!(result.reason.contains("8"));
        println!("Small hole result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_missing_dynamic_input() {
        let expr = Expression::Input(InputSource::Dynamic {
            event: "nonexistent".to_string(),
            field: "Field".to_string(),
        });

        let compiled_paths = vec![(1, "Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);
        let static_data = HashMap::new();
        let dynamic_data = HashMap::new(); // Missing the required event

        let result = evaluator.eval(&static_data, &dynamic_data);
        assert!(result.is_err());

        match result {
            Err(EvaluationError::InputNotFound(name)) => {
                assert!(name.contains("nonexistent"));
                println!("Correctly handled missing dynamic input: {}", name);
            }
            _ => panic!("Expected InputNotFound error"),
        }
    }

    #[test]
    fn test_evaluator_cross_product_evaluation() {
        // Create expression: $hole.Diameter > 5 AND $tear.Length < 20
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

        let compiled_paths = vec![(1, "Complex Defect".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let mut dynamic_data = HashMap::new();

        // Create multiple hole events
        let mut hole1 = HashMap::new();
        hole1.insert("Diameter".to_string(), 3.0); // Too small
        let mut hole2 = HashMap::new();
        hole2.insert("Diameter".to_string(), 8.0); // Good size
        dynamic_data.insert("hole".to_string(), vec![hole1, hole2]);

        // Create tear events
        let mut tear1 = HashMap::new();
        tear1.insert("Length".to_string(), 15.0); // Good length
        let mut tear2 = HashMap::new();
        tear2.insert("Length".to_string(), 25.0); // Too long
        dynamic_data.insert("tear".to_string(), vec![tear1, tear2]);

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Cross-product evaluation should succeed");

        // Should find combination: hole2 (8.0 > 5) AND tear1 (15.0 < 20)
        assert_eq!(result.quality_name, Some("Complex Defect".to_string()));
        assert!(result.reason.contains("hole.Diameter"));
        assert!(result.reason.contains("tear.Length"));
        println!("Cross-product result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_cross_product_no_match() {
        // Expression that should not match any combination
        let expr = Expression::And(
            Box::new(Expression::GreaterThan(
                Box::new(Expression::Input(InputSource::Dynamic {
                    event: "hole".to_string(),
                    field: "Diameter".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(100.0))), // Very large
            )),
            Box::new(Expression::SmallerThan(
                Box::new(Expression::Input(InputSource::Dynamic {
                    event: "tear".to_string(),
                    field: "Length".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(1.0))), // Very small
            )),
        );

        let compiled_paths = vec![(1, "Impossible Defect".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let dynamic_data = create_sample_dynamic_data();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Evaluation should succeed even with no matches");

        assert!(result.quality_name.is_none());
        assert_eq!(result.reason, "No quality triggered");
        println!("No match result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_arithmetic_operations() {
        // Test: $A + $B > 50
        let expr = Expression::GreaterThan(
            Box::new(Expression::Sum(
                Box::new(Expression::Input(InputSource::Static {
                    name: "A".to_string(),
                })),
                Box::new(Expression::Input(InputSource::Static {
                    name: "B".to_string(),
                })),
            )),
            Box::new(Expression::Literal(Value::Number(50.0))),
        );

        let compiled_paths = vec![(1, "Sum Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let mut static_data = HashMap::new();
        static_data.insert("A".to_string(), 30.0);
        static_data.insert("B".to_string(), 25.0); // 30 + 25 = 55 > 50
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Arithmetic evaluation should succeed");

        assert_eq!(result.quality_name, Some("Sum Test".to_string()));
        println!("Arithmetic result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_logical_operations() {
        // Test: $A > 10 OR $B < 5
        let expr = Expression::Or(
            Box::new(Expression::GreaterThan(
                Box::new(Expression::Input(InputSource::Static {
                    name: "A".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(10.0))),
            )),
            Box::new(Expression::SmallerThan(
                Box::new(Expression::Input(InputSource::Static {
                    name: "B".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(5.0))),
            )),
        );

        let compiled_paths = vec![(1, "Logic Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        // Test case where first condition is true
        let mut static_data = HashMap::new();
        static_data.insert("A".to_string(), 15.0); // > 10
        static_data.insert("B".to_string(), 8.0); // > 5
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Logical evaluation should succeed");

        assert_eq!(result.quality_name, Some("Logic Test".to_string()));
        println!("Logical OR result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_short_circuit_and() {
        // Test AND short-circuiting: false AND (anything)
        let expr = Expression::And(
            Box::new(Expression::Literal(Value::Bool(false))),
            Box::new(Expression::Input(InputSource::Static {
                name: "ShouldNotEvaluate".to_string(),
            })),
        );

        let compiled_paths = vec![(1, "Short Circuit Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new(); // Missing ShouldNotEvaluate, but shouldn't matter
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Short-circuit evaluation should succeed");

        assert!(result.quality_name.is_none());
        println!("Short-circuit AND result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_short_circuit_or() {
        // Test OR short-circuiting: true OR (anything)
        let expr = Expression::Or(
            Box::new(Expression::Literal(Value::Bool(true))),
            Box::new(Expression::Input(InputSource::Static {
                name: "ShouldNotEvaluate".to_string(),
            })),
        );

        let compiled_paths = vec![(1, "Short Circuit OR Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new(); // Missing ShouldNotEvaluate, but shouldn't matter
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Short-circuit OR evaluation should succeed");

        assert_eq!(
            result.quality_name,
            Some("Short Circuit OR Test".to_string())
        );
        println!("Short-circuit OR result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_type_mismatch_errors() {
        // Try to perform numeric operation on boolean
        let expr = Expression::Sum(
            Box::new(Expression::Literal(Value::Bool(true))),
            Box::new(Expression::Literal(Value::Number(5.0))),
        );

        let compiled_paths = vec![(1, "Type Error Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let dynamic_data = HashMap::new();

        let result = evaluator.eval(&static_data, &dynamic_data);
        assert!(result.is_err());

        match result {
            Err(EvaluationError::TypeMismatch { expected, found }) => {
                assert_eq!(expected, "Number");
                println!(
                    "Correctly handled type mismatch: expected {}, found {}",
                    expected, found
                );
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_evaluator_priority_ordering() {
        // Create multiple qualities with different priorities
        let compiled_paths = vec![
            (
                3,
                "Low Priority".to_string(),
                Expression::Literal(Value::Bool(true)),
            ),
            (
                1,
                "High Priority".to_string(),
                Expression::Literal(Value::Bool(true)),
            ),
            (
                2,
                "Medium Priority".to_string(),
                Expression::Literal(Value::Bool(true)),
            ),
        ];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Priority evaluation should succeed");

        // Should return the highest priority (lowest number) that evaluates to true
        assert_eq!(result.quality_name, Some("High Priority".to_string()));
        assert_eq!(result.quality_priority, Some(1));
        println!("Priority test result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_trace_formatting() {
        let expr = Expression::GreaterThan(
            Box::new(Expression::Input(InputSource::Static {
                name: "Temperature".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(25.0))),
        );

        let compiled_paths = vec![(1, "Trace Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let mut static_data = HashMap::new();
        static_data.insert("Temperature".to_string(), 30.5);
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Trace evaluation should succeed");

        // Check that trace contains expected elements
        assert!(result.reason.contains("Temperature"));
        assert!(result.reason.contains("30.5") || result.reason.contains("30"));
        assert!(result.reason.contains(">"));
        assert!(result.reason.contains("25"));
        println!("Trace formatting result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_complex_nested_expressions() {
        // Complex nested expression: ((A > 10) AND (B < 5)) OR (C == 42)
        let expr = Expression::Or(
            Box::new(Expression::And(
                Box::new(Expression::GreaterThan(
                    Box::new(Expression::Input(InputSource::Static {
                        name: "A".to_string(),
                    })),
                    Box::new(Expression::Literal(Value::Number(10.0))),
                )),
                Box::new(Expression::SmallerThan(
                    Box::new(Expression::Input(InputSource::Static {
                        name: "B".to_string(),
                    })),
                    Box::new(Expression::Literal(Value::Number(5.0))),
                )),
            )),
            Box::new(Expression::Equal(
                Box::new(Expression::Input(InputSource::Static {
                    name: "C".to_string(),
                })),
                Box::new(Expression::Literal(Value::Number(42.0))),
            )),
        );

        let compiled_paths = vec![(1, "Complex Test".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        // Test case where only the second part of OR is true
        let mut static_data = HashMap::new();
        static_data.insert("A".to_string(), 5.0); // <= 10
        static_data.insert("B".to_string(), 10.0); // >= 5
        static_data.insert("C".to_string(), 42.0); // == 42
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Complex evaluation should succeed");

        assert_eq!(result.quality_name, Some("Complex Test".to_string()));
        println!("Complex nested result: {}", result.reason);
    }

    #[test]
    fn test_evaluator_multiple_event_instances() {
        // Test with multiple instances of the same event type
        let expr = Expression::SmallerThan(
            Box::new(Expression::Input(InputSource::Dynamic {
                event: "hole".to_string(),
                field: "Diameter".to_string(),
            })),
            Box::new(Expression::Literal(Value::Number(8.0))),
        );

        let compiled_paths = vec![(1, "Small Hole".to_string(), expr)];

        let evaluator = Evaluator::new(compiled_paths);

        let static_data = HashMap::new();
        let mut dynamic_data = HashMap::new();

        // Create multiple hole instances - some matching, some not
        let mut hole1 = HashMap::new();
        hole1.insert("Diameter".to_string(), 12.0); // Too big

        let mut hole2 = HashMap::new();
        hole2.insert("Diameter".to_string(), 6.0); // Just right

        let mut hole3 = HashMap::new();
        hole3.insert("Diameter".to_string(), 10.0); // Too big

        dynamic_data.insert("hole".to_string(), vec![hole1, hole2, hole3]);

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Multiple instances evaluation should succeed");

        // Should match the second hole (6.0 < 8.0)
        assert_eq!(result.quality_name, Some("Small Hole".to_string()));
        assert!(result.reason.contains("6"));
        println!("Multiple instances result: {}", result.reason);
    }
}
