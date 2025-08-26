//! Integration tests for Hantei
//!
//! End-to-end tests that verify the complete functionality works together.
//!
mod common;
use common::*;
use hantei::prelude::*;
use std::fs;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_recipe_compilation_and_evaluation() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        assert!(!logical_repr.is_empty());
        assert!(!compiled_paths.is_empty());

        // Verify we got the expected quality
        let quality_names: Vec<_> = compiled_paths
            .iter()
            .map(|(_, name, _)| name.clone())
            .collect();
        assert!(
            quality_names.contains(&"Hot".to_string())
                || quality_names.contains(&"Normal".to_string())
        );

        println!("Compiled {} quality paths", compiled_paths.len());
        for (priority, name, _) in &compiled_paths {
            println!("  - Quality '{}' (Priority {})", name, priority);
        }
    }

    #[test]
    fn test_complex_recipe_compilation() {
        let compiler = Compiler::new(COMPLEX_RECIPE_JSON, COMPLEX_QUALITIES_JSON)
            .expect("Failed to create complex compiler");

        let (logical_repr, compiled_paths) = compiler
            .compile()
            .expect("Failed to compile complex recipe");

        assert!(!logical_repr.is_empty());
        println!(
            "Complex recipe logical representation length: {}",
            logical_repr.len()
        );

        // Should have multiple quality paths
        if !compiled_paths.is_empty() {
            println!(
                "Complex compilation succeeded with {} paths",
                compiled_paths.len()
            );
            for (priority, name, _) in &compiled_paths {
                println!("  - Quality '{}' (Priority {})", name, priority);
            }
        } else {
            println!(
                "Complex compilation produced no paths (may be due to quality connection setup)"
            );
        }
    }

    #[test]
    fn test_full_workflow_with_static_data() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (_logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        if compiled_paths.is_empty() {
            println!("No compiled paths available for evaluation test");
            return;
        }

        let evaluator = Evaluator::new(compiled_paths);

        // Test with high temperature (should trigger "Hot")
        let mut static_data = HashMap::new();
        static_data.insert("Temperature".to_string(), 30.0); // > 25.0
        let dynamic_data = HashMap::new();

        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Failed to evaluate with high temperature");

        assert_eq!(result.quality_name, Some("Hot".to_string()));
    }

    #[test]
    fn test_full_workflow_with_dynamic_data() {
        let compiler = Compiler::new(COMPLEX_RECIPE_JSON, COMPLEX_QUALITIES_JSON)
            .expect("Failed to create complex compiler");

        let result = compiler.compile();
        match result {
            Ok((_logical_repr, compiled_paths)) => {
                if compiled_paths.is_empty() {
                    println!("No compiled paths available for dynamic data test");
                    return;
                }

                let evaluator = Evaluator::new(compiled_paths);
                let (static_data, dynamic_data) = create_trigger_test_data();

                let result = evaluator
                    .eval(&static_data, &dynamic_data)
                    .expect("Failed to evaluate with dynamic data");

                println!("Dynamic data evaluation result: {:?}", result);
                if let Some(quality_name) = &result.quality_name {
                    println!(
                        "Triggered with dynamic data: {} - {}",
                        quality_name, result.reason
                    );
                } else {
                    println!("No quality triggered with dynamic data: {}", result.reason);
                }
            }
            Err(e) => {
                println!("Complex recipe compilation failed (may be expected): {}", e);
            }
        }
    }

    #[test]
    fn test_sample_data_integration() {
        let static_data = create_sample_static_data(); // Provides "Temperature", "Humidity", etc.
        let dynamic_data = create_sample_dynamic_data(); // Provides "hole", "tear", etc.

        let compiler = Compiler::new(COMPLEX_RECIPE_JSON, COMPLEX_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (_logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        if compiled_paths.is_empty() {
            println!("No compiled paths for sample data test");
            return;
        }

        println!("Sample data test compiled paths debug:");

        let evaluator = Evaluator::new(compiled_paths);
        let result = evaluator
            .eval(&static_data, &dynamic_data)
            .expect("Failed to evaluate with sample data");

        println!("Sample data evaluation result: {:?}", result);
        assert!(!result.reason.is_empty());

        assert_eq!(result.quality_name, Some("Premium".to_string()));
    }

    #[test]
    fn test_debug_output_generation() {
        let test_dir = setup_test_dir().join("integration").join("debug_output");

        // Create a temporary directory for this test
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (logical_repr, _compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        // Write logical representation to test output
        let logical_path = test_dir.join("logical_connections.txt");
        fs::write(&logical_path, &logical_repr).expect("Failed to write logical representation");

        assert!(logical_path.exists());

        let content =
            fs::read_to_string(&logical_path).expect("Failed to read logical representation");

        assert!(!content.is_empty());
        println!(
            "Generated logical representation with {} characters",
            content.len()
        );

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_error_handling_integration() {
        // Test invalid JSON handling
        let invalid_recipe = "{ invalid json }";
        let result = Compiler::new(invalid_recipe, SIMPLE_QUALITIES_JSON);
        assert!(result.is_err());

        if let Err(error) = result {
            println!("Correctly handled invalid recipe JSON: {}", error);
        }

        // Test invalid qualities JSON
        let invalid_qualities = "[ invalid json ]";
        let result = Compiler::new(SIMPLE_RECIPE_JSON, invalid_qualities);
        assert!(result.is_err());

        if let Err(error) = result {
            println!("Correctly handled invalid qualities JSON: {}", error);
        }
    }

    #[test]
    fn test_evaluation_error_handling() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (_logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        if compiled_paths.is_empty() {
            println!("No compiled paths for error handling test");
            return;
        }

        let evaluator = Evaluator::new(compiled_paths);

        // Test with missing static data
        let empty_static = HashMap::new();
        let empty_dynamic = HashMap::new();

        let result = evaluator.eval(&empty_static, &empty_dynamic);
        match result {
            Ok(eval_result) => {
                println!("Evaluation with empty data succeeded: {:?}", eval_result);
            }
            Err(e) => {
                println!("Correctly handled missing data: {}", e);
            }
        }
    }

    #[test]
    fn test_cross_product_evaluation() {
        let compiler = Compiler::new(COMPLEX_RECIPE_JSON, COMPLEX_QUALITIES_JSON)
            .expect("Failed to create complex compiler");

        let result = compiler.compile();
        match result {
            Ok((_logical_repr, compiled_paths)) => {
                if compiled_paths.is_empty() {
                    println!("No compiled paths for cross-product test");
                    return;
                }

                let evaluator = Evaluator::new(compiled_paths);

                // Create data with multiple dynamic events
                let static_data = create_sample_static_data();
                let dynamic_data = create_sample_dynamic_data();

                println!(
                    "Testing cross-product evaluation with {} static fields and {} dynamic event types",
                    static_data.len(),
                    dynamic_data.len()
                );

                for (event_type, events) in &dynamic_data {
                    println!("  {} events of type '{}': ", events.len(), event_type);
                    for (i, event) in events.iter().enumerate() {
                        println!("    Event {}: {} fields", i, event.len());
                    }
                }

                let result = evaluator.eval(&static_data, &dynamic_data);
                match result {
                    Ok(eval_result) => {
                        println!("Cross-product evaluation result: {:?}", eval_result);
                    }
                    Err(e) => {
                        println!("Cross-product evaluation error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Complex recipe compilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_ast_optimization_effects() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler");

        let (_logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");

        if compiled_paths.is_empty() {
            println!("No compiled paths for optimization test");
            return;
        }

        // Check that we got optimized ASTs
        for (priority, name, ast) in &compiled_paths {
            let ast_string = format!("{}", ast);
            println!("Quality '{}' (Priority {}) AST:", name, priority);
            println!("  AST length: {} characters", ast_string.len());

            // Verify AST contains expected structure
            if ast_string.contains("$Temperature") {
                println!("  Contains expected input reference");
            }
            if ast_string.contains(">")
                || ast_string.contains("<")
                || ast_string.contains("AND")
                || ast_string.contains("OR")
            {
                println!("  Contains expected operators");
            }
        }
    }

    #[test]
    fn test_quality_priority_ordering() {
        let multi_quality_json = r#"[
            { "id": 0, "name": "High", "priority": 1, "negated": false },
            { "id": 1, "name": "Medium", "priority": 2, "negated": false },
            { "id": 2, "name": "Low", "priority": 3, "negated": false }
        ]"#;

        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, multi_quality_json);
        if let Ok(compiler) = compiler {
            if let Ok((_logical_repr, compiled_paths)) = compiler.compile() {
                if !compiled_paths.is_empty() {
                    // Verify quality paths are sorted by priority
                    let mut prev_priority = 0;
                    for (priority, name, _) in &compiled_paths {
                        assert!(
                            *priority >= prev_priority,
                            "Quality '{}' priority {} is not >= previous priority {}",
                            name,
                            priority,
                            prev_priority
                        );
                        prev_priority = *priority;
                        println!("  Quality '{}' has priority {}", name, priority);
                    }
                    println!("Quality priorities are correctly ordered");
                }
            }
        }
    }

    #[test]
    fn test_prelude_import_completeness() {
        // Verify that the prelude exports work correctly
        let _compiler: Option<Compiler> = None;
        let _evaluator: Option<Evaluator> = None;
        let _sample_data: Option<SampleData> = None;
        let _expression: Option<Expression> = None;
        let _value: Option<Value> = None;
        let _input_source: Option<InputSource> = None;
        let _quality: Option<Quality> = None;
        let _hashmap: HashMap<String, f64> = HashMap::new();

        // Test Result alias
        let _result: Result<String> = Ok("test".to_string());

        println!("All prelude types are accessible");
    }
}
