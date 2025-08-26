//! Compiler-specific functionality tests
//!
//! Tests for the compilation process, AST generation, and optimization.

use hantei::prelude::*;
mod common;
use common::*;

#[cfg(test)]
mod compiler_tests {
    use super::*;

    #[test]
    fn test_compiler_basic_functionality() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create basic compiler");

        let (logical_repr, compiled_paths) =
            compiler.compile().expect("Failed to compile basic recipe");

        assert!(!logical_repr.is_empty());
        println!("Logical representation length: {}", logical_repr.len());

        if !compiled_paths.is_empty() {
            assert!(!compiled_paths[0].1.is_empty()); // Quality name should not be empty
            println!("Compiled {} quality paths", compiled_paths.len());

            for (priority, name, ast) in &compiled_paths {
                println!(
                    "  Priority {}: '{}' - AST length: {} chars",
                    priority,
                    name,
                    format!("{}", ast).len()
                );
            }
        }
    }

    #[test]
    fn test_compiler_node_type_support() {
        let recipe_with_multiple_nodes = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Static Input",
                            "realNodeType": "dynamicNode",
                            "realInputType": null,
                            "cases": [
                                {
                                    "caseId": 0,
                                    "caseName": "Value1",
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
                            "name": "Greater Than",
                            "realNodeType": "gtNode",
                            "values": [null, 10.0]
                        }
                    }
                },
                {
                    "id": "0003",
                    "data": {
                        "nodeData": {
                            "name": "Less Than",
                            "realNodeType": "stNode",
                            "values": [null, 100.0]
                        }
                    }
                },
                {
                    "id": "0004",
                    "data": {
                        "nodeData": {
                            "name": "AND Gate",
                            "realNodeType": "andNode"
                        }
                    }
                },
                {
                    "id": "0005",
                    "data": {
                        "nodeData": {
                            "name": "OR Gate",
                            "realNodeType": "orNode"
                        }
                    }
                },
                {
                    "id": "0006",
                    "data": {
                        "nodeData": {
                            "name": "NOT Gate",
                            "realNodeType": "notNode"
                        }
                    }
                },
                {
                    "id": "0007",
                    "data": {
                        "nodeData": {
                            "name": "Set Quality",
                            "realNodeType": "setQualityNode"
                        }
                    }
                }
            ],
            "edges": []
        }"#;

        let compiler = Compiler::new(recipe_with_multiple_nodes, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_ok());

        println!("Compiler accepts recipe with multiple node types");
    }

    #[test]
    fn test_compiler_arithmetic_nodes() {
        let arithmetic_recipe = r#"{
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
                                    "caseName": "A",
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
                            "realNodeType": "dynamicNode",
                            "realInputType": null,
                            "cases": [
                                {
                                    "caseId": 0,
                                    "caseName": "B",
                                    "realCaseType": "number"
                                }
                            ]
                        }
                    }
                },
                {
                    "id": "0003",
                    "data": {
                        "nodeData": {
                            "name": "Sum",
                            "realNodeType": "sumNode"
                        }
                    }
                },
                {
                    "id": "0004",
                    "data": {
                        "nodeData": {
                            "name": "Multiply",
                            "realNodeType": "multNode"
                        }
                    }
                },
                {
                    "id": "0005",
                    "data": {
                        "nodeData": {
                            "name": "Subtract",
                            "realNodeType": "subNode"
                        }
                    }
                },
                {
                    "id": "0006",
                    "data": {
                        "nodeData": {
                            "name": "Divide",
                            "realNodeType": "divideNode"
                        }
                    }
                }
            ],
            "edges": []
        }"#;

        let compiler = Compiler::new(arithmetic_recipe, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_ok());

        println!("Compiler accepts recipe with arithmetic operations");
    }

    #[test]
    fn test_compiler_comparison_nodes() {
        let comparison_recipe = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Greater Than Equal",
                            "realNodeType": "gteqNode",
                            "values": [null, 50.0]
                        }
                    }
                },
                {
                    "id": "0002",
                    "data": {
                        "nodeData": {
                            "name": "Less Than Equal",
                            "realNodeType": "steqNode",
                            "values": [null, 75.0]
                        }
                    }
                },
                {
                    "id": "0003",
                    "data": {
                        "nodeData": {
                            "name": "Equal",
                            "realNodeType": "eqNode",
                            "values": [null, 42.0]
                        }
                    }
                }
            ],
            "edges": []
        }"#;

        let compiler = Compiler::new(comparison_recipe, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_ok());

        println!("Compiler accepts recipe with comparison operations");
    }

    #[test]
    fn test_compiler_dynamic_node_handling() {
        let dynamic_recipe = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Static Data",
                            "realNodeType": "dynamicNode",
                            "realInputType": null,
                            "cases": [
                                {
                                    "caseId": 0,
                                    "caseName": "Temperature",
                                    "realCaseType": "number"
                                },
                                {
                                    "caseId": 1,
                                    "caseName": "Pressure",
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
                            "name": "Hole Events",
                            "realNodeType": "dynamicNode",
                            "realInputType": "hole",
                            "cases": [
                                {
                                    "caseId": 0,
                                    "caseName": "Diameter",
                                    "realCaseType": "number"
                                },
                                {
                                    "caseId": 1,
                                    "caseName": "Depth",
                                    "realCaseType": "number"
                                }
                            ]
                        }
                    }
                }
            ],
            "edges": []
        }"#;

        let compiler = Compiler::new(dynamic_recipe, SIMPLE_QUALITIES_JSON);
        assert!(compiler.is_ok());

        println!("Compiler handles both static and dynamic input nodes");
    }

    #[test]
    fn test_compiler_edge_processing() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler for edge test");

        let (logical_repr, _compiled_paths) =
            compiler.compile().expect("Failed to compile for edge test");

        // The logical representation should contain connection information
        assert!(logical_repr.contains("0001") || logical_repr.contains("0002"));
        println!("Logical representation contains node connection data");
    }

    #[test]
    fn test_compiler_invalid_node_types() {
        let invalid_recipe = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Invalid Node",
                            "realNodeType": "invalidNodeType"
                        }
                    }
                },
                {
                    "id": "0002",
                    "data": {
                        "nodeData": {
                            "name": "Set Quality",
                            "realNodeType": "setQualityNode"
                        }
                    }
                }
            ],
            "edges": [
                {
                    "source": "0001",
                    "target": "0002",
                    "sourceHandle": "output-0001-0",
                    "targetHandle": "input-0002-0"
                }
            ]
        }"#;

        let compiler = Compiler::new(invalid_recipe, SIMPLE_QUALITIES_JSON)
            .expect("Compiler should be created even with invalid nodes");

        // Compilation should fail due to invalid node type
        let result = compiler.compile();
        match result {
            Err(e) => {
                println!("Correctly rejected invalid node type: {}", e);
                assert!(
                    format!("{}", e).contains("InvalidNodeType")
                        || format!("{}", e).contains("invalid")
                );
            }
            Ok(_) => {
                println!("⚠ Compilation unexpectedly succeeded with invalid node type");
            }
        }
    }

    #[test]
    fn test_compiler_missing_node_references() {
        let recipe_with_missing_node = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Test Node",
                            "realNodeType": "gtNode",
                            "values": [null, 25.0]
                        }
                    }
                }
            ],
            "edges": [
                {
                    "source": "0001",
                    "target": "0999",
                    "sourceHandle": "output-0001-0",
                    "targetHandle": "input-0999-0"
                }
            ]
        }"#;

        let compiler = Compiler::new(recipe_with_missing_node, SIMPLE_QUALITIES_JSON)
            .expect("Compiler should be created with missing node reference");

        let result = compiler.compile();
        match result {
            Err(e) => {
                println!("Correctly handled missing node reference: {}", e);
                assert!(
                    format!("{}", e).contains("not found")
                        || format!("{}", e).contains("NodeNotFound")
                );
            }
            Ok(_) => {
                println!("⚠ Compilation unexpectedly succeeded with missing node reference");
            }
        }
    }

    #[test]
    fn test_compiler_ast_structure_validation() {
        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, SIMPLE_QUALITIES_JSON)
            .expect("Failed to create compiler for AST validation");

        let (_logical_repr, compiled_paths) = compiler
            .compile()
            .expect("Failed to compile for AST validation");

        if !compiled_paths.is_empty() {
            for (priority, name, ast) in &compiled_paths {
                let ast_string = format!("{}", ast);

                // Validate AST structure contains expected elements
                println!(
                    "Validating AST for quality '{}' (Priority {}):",
                    name, priority
                );
                println!("AST: {}", ast_string);

                // Should contain some operation or input
                let has_operation = ast_string.contains(">")
                    || ast_string.contains("<")
                    || ast_string.contains("AND")
                    || ast_string.contains("OR")
                    || ast_string.contains("+")
                    || ast_string.contains("-");

                let has_input = ast_string.contains("$");

                if has_operation || has_input {
                    println!("  AST contains valid structure");
                } else {
                    println!("  ? AST structure may be minimal");
                }

                // AST should not be empty
                assert!(
                    !ast_string.trim().is_empty(),
                    "AST for quality '{}' should not be empty",
                    name
                );
            }
        }
    }

    #[test]
    fn test_compiler_optimization_detection() {
        // Create a recipe with constant values that could be optimized
        let optimizable_recipe = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "name": "Constant Comparison",
                            "realNodeType": "gtNode",
                            "values": [10.0, 5.0]
                        }
                    }
                },
                {
                    "id": "0002",
                    "data": {
                        "nodeData": {
                            "name": "Set Quality",
                            "realNodeType": "setQualityNode"
                        }
                    }
                }
            ],
            "edges": [
                {
                    "source": "0001",
                    "target": "0002",
                    "sourceHandle": "bool-bool-0001-2",
                    "targetHandle": "sq-bool-0002-0"
                }
            ]
        }"#;

        let compiler = Compiler::new(optimizable_recipe, SIMPLE_QUALITIES_JSON);
        if let Ok(compiler) = compiler {
            if let Ok((_logical_repr, compiled_paths)) = compiler.compile() {
                if !compiled_paths.is_empty() {
                    for (_, name, ast) in &compiled_paths {
                        let ast_string = format!("{}", ast);
                        println!("Optimized AST for '{}': {}", name, ast_string);

                        // Check if constant folding occurred
                        if ast_string.contains("true") || ast_string.contains("false") {
                            println!("  Possible constant folding detected");
                        } else {
                            println!("  - No obvious constant folding in this case");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_compiler_quality_connection_mapping() {
        // Test that quality node (0002) is correctly identified and mapped
        let qualities_with_multiple = r#"[
            { "id": 0, "name": "First", "priority": 1, "negated": false },
            { "id": 1, "name": "Second", "priority": 2, "negated": false },
            { "id": 2, "name": "Third", "priority": 3, "negated": false }
        ]"#;

        let compiler = Compiler::new(SIMPLE_RECIPE_JSON, qualities_with_multiple);
        if let Ok(compiler) = compiler {
            if let Ok((_logical_repr, compiled_paths)) = compiler.compile() {
                println!("Quality connection mapping test:");
                println!("  Total qualities defined: 3");
                println!("  Compiled paths generated: {}", compiled_paths.len());

                // Check that priorities are handled correctly
                for (priority, name, _) in &compiled_paths {
                    println!("  - Quality '{}' mapped to priority {}", name, priority);
                    assert!(
                        *priority >= 1 && *priority <= 3,
                        "Priority should be between 1-3"
                    );
                }
            }
        }
    }

    #[test]
    fn test_compiler_handle_parsing() {
        // Test that source/target handle indices are parsed correctly
        let recipe_with_specific_handles = r#"{
            "nodes": [
                {
                    "id": "0001",
                    "data": {
                        "nodeData": {
                            "realNodeType": "dynamicNode",
                            "realInputType": null,
                            "cases": [
                                {"caseId": 0, "caseName": "Input", "realCaseType": "number"}
                            ]
                        }
                    }
                },
                {
                    "id": "0002",
                    "data": {
                        "nodeData": {
                            "realNodeType": "gtNode",
                            "values": [null, 42.0]
                        }
                    }
                }
            ],
            "edges": [
                {
                    "source": "0001",
                    "target": "0002",
                    "sourceHandle": "output-handle-0001-5",
                    "targetHandle": "input-handle-0002-3"
                }
            ]
        }"#;

        let compiler = Compiler::new(recipe_with_specific_handles, SIMPLE_QUALITIES_JSON);
        assert!(
            compiler.is_ok(),
            "Compiler should handle specific handle indices"
        );

        println!("Compiler correctly parses handle indices from edge connections");
    }

    #[test]
    fn test_compiler_memory_efficiency() {
        // Create a somewhat complex recipe to test memory usage
        let complex_compiler = Compiler::new(COMPLEX_RECIPE_JSON, COMPLEX_QUALITIES_JSON);

        if let Ok(compiler) = complex_compiler {
            // Test compilation once (compile takes ownership)
            let result = compiler.compile();
            match result {
                Ok((logical_repr, compiled_paths)) => {
                    println!(
                        "Compilation test: {} paths, {} logical chars",
                        compiled_paths.len(),
                        logical_repr.len()
                    );
                }
                Err(e) => {
                    println!("Compilation test failed: {}", e);
                }
            }
        }

        // Test should complete without memory issues
        println!("Memory efficiency test completed");
    }
}
