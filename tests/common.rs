//! Common test utilities and constants
//!
//! Shared functionality used across all test modules for hantei.

use std::collections::HashMap;
use std::path::PathBuf;

#[allow(dead_code)]
pub const TEST_OUTPUT_DIR: &str = "tests/tmp";
#[allow(dead_code)]
pub const TEST_TIMEOUT_MS: u64 = 5000;

// Sample recipe JSON for testing
#[allow(dead_code)]
pub const SIMPLE_RECIPE_JSON: &str = r#"{
    "nodes": [
        {
            "id": "0001",
            "data": {
                "nodeData": {
                    "name": "Temperature Input",
                    "realNodeType": "dynamicNode",
                    "realInputType": null,
                    "cases": [
                        {
                            "caseId": 0,
                            "caseName": "Temperature",
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
                    "name": "Greater than 25",
                    "realNodeType": "gtNode",
                    "values": [null, 25.0]
                }
            }
        },
        {
            "id": "0003",
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
            "sourceHandle": "number-number-0001-0",
            "targetHandle": "number-number-0002-0"
        },
        {
            "source": "0002",
            "target": "0003",
            "sourceHandle": "bool-bool-0002-2",
            "targetHandle": "sq-bool-0003-0"
        }
    ]
}"#;

// Sample qualities JSON for testing
#[allow(dead_code)]
pub const SIMPLE_QUALITIES_JSON: &str = r#"[
    { "id": 0, "name": "Hot", "priority": 1, "negated": false },
    { "id": 1, "name": "Normal", "priority": 2, "negated": false }
]"#;

// Complex recipe with multiple node types
#[allow(dead_code)]
pub const COMPLEX_RECIPE_JSON: &str = r#"{
    "nodes": [
        {
            "id": "0001",
            "data": {
                "nodeData": {
                    "name": "Start",
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
                            "caseName": "Humidity",
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
                            "caseName": "Length",
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
                    "name": "Temperature Check",
                    "realNodeType": "gtNode",
                    "values": [null, 30.0]
                }
            }
        },
        {
            "id": "0004",
            "data": {
                "nodeData": {
                    "name": "Diameter Check",
                    "realNodeType": "stNode",
                    "values": [null, 10.0]
                }
            }
        },
        {
            "id": "0005",
            "data": {
                "nodeData": {
                    "name": "Combine Conditions",
                    "realNodeType": "andNode"
                }
            }
        },
        {
            "id": "0006",
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
            "target": "0003",
            "sourceHandle": "number-number-0001-0",
            "targetHandle": "number-number-0003-0"
        },
        {
            "source": "0002",
            "target": "0004",
            "sourceHandle": "number-number-0002-0",
            "targetHandle": "number-number-0004-0"
        },
        {
            "source": "0003",
            "target": "0005",
            "sourceHandle": "bool-bool-0003-2",
            "targetHandle": "bool-bool-0005-0"
        },
        {
            "source": "0004",
            "target": "0005",
            "sourceHandle": "bool-bool-0004-2",
            "targetHandle": "bool-bool-0005-1"
        },
        {
            "source": "0005",
            "target": "0006",
            "sourceHandle": "bool-bool-0005-2",
            "targetHandle": "sq-bool-0006-0"
        }
    ]
}"#;

// Complex qualities with multiple priorities
#[allow(dead_code)]
pub const COMPLEX_QUALITIES_JSON: &str = r#"[
    { "id": 0, "name": "Premium", "priority": 1, "negated": false },
    { "id": 1, "name": "Standard", "priority": 2, "negated": false },
    { "id": 2, "name": "Defective", "priority": 3, "negated": false }
]"#;

/// Helper function to create test output directory
#[allow(dead_code)]
pub fn setup_test_dir() -> PathBuf {
    let test_dir = PathBuf::from(TEST_OUTPUT_DIR);
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).unwrap();
    }

    // Create organized subdirectories for different test types
    let subdirs = [
        "unit/compiler",
        "unit/evaluator",
        "unit/ast",
        "unit/data",
        "integration/full_workflow",
        "integration/edge_cases",
    ];

    for subdir in &subdirs {
        let dir_path = test_dir.join(subdir);
        if !dir_path.exists() {
            let _ = std::fs::create_dir_all(&dir_path);
        }
    }

    test_dir
}

/// Helper function to clean up test files
#[allow(dead_code)]
pub fn cleanup_test_dir() {
    let test_dir = PathBuf::from(TEST_OUTPUT_DIR);
    if test_dir.exists() {
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

/// Helper function to clean up only a specific test subdirectory
#[allow(dead_code)]
pub fn cleanup_test_subdir(subdir: &str) {
    let test_dir = PathBuf::from(TEST_OUTPUT_DIR).join(subdir);
    if test_dir.exists() {
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

/// Create sample static data for testing
#[allow(dead_code)]
pub fn create_sample_static_data() -> HashMap<String, f64> {
    let mut data = HashMap::new();
    data.insert("Temperature".to_string(), 32.5);
    data.insert("Humidity".to_string(), 65.0);
    data.insert("Pressure".to_string(), 1013.25);
    data
}

/// Create sample dynamic data for testing
#[allow(dead_code)]
pub fn create_sample_dynamic_data() -> HashMap<String, Vec<HashMap<String, f64>>> {
    let mut data = HashMap::new();

    // Hole events
    let mut hole_events = Vec::new();
    let mut hole1 = HashMap::new();
    hole1.insert("Diameter".to_string(), 5.2);
    hole1.insert("Length".to_string(), 12.0);
    hole_events.push(hole1);

    let mut hole2 = HashMap::new();
    hole2.insert("Diameter".to_string(), 8.7);
    hole2.insert("Length".to_string(), 15.5);
    hole_events.push(hole2);

    data.insert("hole".to_string(), hole_events);

    // Tear events
    let mut tear_events = Vec::new();
    let mut tear1 = HashMap::new();
    tear1.insert("Length".to_string(), 25.0);
    tear1.insert("Width".to_string(), 2.1);
    tear_events.push(tear1);

    data.insert("tear".to_string(), tear_events);

    data
}

/// Create minimal test data for edge cases
#[allow(dead_code)]
pub fn create_minimal_test_data() -> (
    HashMap<String, f64>,
    HashMap<String, Vec<HashMap<String, f64>>>,
) {
    let mut static_data = HashMap::new();
    static_data.insert("Temperature".to_string(), 20.0);

    let dynamic_data = HashMap::new();

    (static_data, dynamic_data)
}

/// Create test data that should trigger specific conditions
#[allow(dead_code)]
pub fn create_trigger_test_data() -> (
    HashMap<String, f64>,
    HashMap<String, Vec<HashMap<String, f64>>>,
) {
    let mut static_data = HashMap::new();
    static_data.insert("Temperature".to_string(), 35.0); // > 30.0

    let mut dynamic_data = HashMap::new();

    // Small diameter hole
    let mut hole_events = Vec::new();
    let mut hole1 = HashMap::new();
    hole1.insert("Diameter".to_string(), 8.0); // < 10.0
    hole1.insert("Length".to_string(), 12.0);
    hole_events.push(hole1);

    dynamic_data.insert("hole".to_string(), hole_events);

    (static_data, dynamic_data)
}

/// Validate that AST string representation contains expected patterns
#[allow(dead_code)]
pub fn validate_ast_structure(ast_string: &str, expected_patterns: &[&str]) -> bool {
    for pattern in expected_patterns {
        if !ast_string.contains(pattern) {
            return false;
        }
    }
    true
}

/// Helper to count occurrences of a pattern in AST string
#[allow(dead_code)]
pub fn count_ast_patterns(ast_string: &str, pattern: &str) -> usize {
    ast_string.matches(pattern).count()
}
