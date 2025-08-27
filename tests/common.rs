//! Common test utilities for building recipe definitions and data.
use hantei::prelude::*;

/// Creates a simple, valid `FlowDefinition` for basic tests.
///
/// Logic: `$Temperature > 25.0` -> Quality 0
#[allow(dead_code)]
pub fn create_simple_flow() -> FlowDefinition {
    FlowDefinition {
        nodes: vec![
            // Static data source node
            FlowNodeDefinition {
                id: "0001".to_string(),
                operation_type: "dynamicNode".to_string(),
                input_type: None, // `None` signifies static data
                literal_values: None,
                data_fields: Some(vec![DataFieldDefinition {
                    id: 0,
                    name: "Temperature".to_string(),
                    data_type: Some("number".to_string()),
                }]),
            },
            // Greater Than node
            FlowNodeDefinition {
                id: "0002".to_string(),
                operation_type: "gtNode".to_string(),
                input_type: None,
                literal_values: Some(vec![serde_json::Value::Null, serde_json::json!(25.0)]),
                data_fields: None,
            },
            // Quality sink node
            FlowNodeDefinition {
                id: "0003".to_string(),
                operation_type: "setQualityNode".to_string(),
                input_type: None,
                literal_values: None,
                data_fields: None,
            },
        ],
        edges: vec![
            FlowEdgeDefinition {
                source: "0001".to_string(),
                target: "0002".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-0".to_string(),
            },
            FlowEdgeDefinition {
                source: "0002".to_string(),
                target: "0003".to_string(),
                source_handle: "output-0".to_string(), // Assumes boolean output is handle 0
                target_handle: "input-0".to_string(),  // Connects to the first quality
            },
        ],
    }
}

/// Creates a simple list of qualities for testing.
#[allow(dead_code)]
pub fn create_simple_qualities() -> Vec<Quality> {
    vec![
        Quality {
            name: "Hot".to_string(),
            priority: 1,
        },
        Quality {
            name: "Normal".to_string(),
            priority: 2,
        },
    ]
}

/// Creates a more complex `FlowDefinition` involving static and dynamic data.
///
/// Logic: `$Temperature > 30.0 AND $hole.Diameter < 10.0` -> Quality 0
#[allow(dead_code)]
pub fn create_complex_flow() -> FlowDefinition {
    FlowDefinition {
        nodes: vec![
            // Static data source
            FlowNodeDefinition {
                id: "static_source".to_string(),
                operation_type: "dynamicNode".to_string(),
                input_type: None,
                literal_values: None,
                data_fields: Some(vec![DataFieldDefinition {
                    id: 0,
                    name: "Temperature".to_string(),
                    data_type: Some("number".to_string()),
                }]),
            },
            // Dynamic data source for "hole" events
            FlowNodeDefinition {
                id: "hole_source".to_string(),
                operation_type: "dynamicNode".to_string(),
                input_type: Some("hole".to_string()),
                literal_values: None,
                data_fields: Some(vec![DataFieldDefinition {
                    id: 0,
                    name: "Diameter".to_string(),
                    data_type: Some("number".to_string()),
                }]),
            },
            // Temp check > 30.0
            FlowNodeDefinition {
                id: "temp_check".to_string(),
                operation_type: "gtNode".to_string(),
                input_type: None,
                literal_values: Some(vec![serde_json::Value::Null, serde_json::json!(30.0)]),
                data_fields: None,
            },
            // Diameter check < 10.0
            FlowNodeDefinition {
                id: "diameter_check".to_string(),
                operation_type: "stNode".to_string(),
                input_type: None,
                literal_values: Some(vec![serde_json::Value::Null, serde_json::json!(10.0)]),
                data_fields: None,
            },
            // AND gate
            FlowNodeDefinition {
                id: "and_gate".to_string(),
                operation_type: "andNode".to_string(),
                input_type: None,
                literal_values: None,
                data_fields: None,
            },
            // Quality sink
            FlowNodeDefinition {
                id: "quality_sink".to_string(),
                operation_type: "setQualityNode".to_string(),
                input_type: None,
                literal_values: None,
                data_fields: None,
            },
        ],
        edges: vec![
            FlowEdgeDefinition {
                source: "static_source".to_string(),
                target: "temp_check".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-0".to_string(),
            },
            FlowEdgeDefinition {
                source: "hole_source".to_string(),
                target: "diameter_check".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-0".to_string(),
            },
            FlowEdgeDefinition {
                source: "temp_check".to_string(),
                target: "and_gate".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-0".to_string(),
            },
            FlowEdgeDefinition {
                source: "diameter_check".to_string(),
                target: "and_gate".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-1".to_string(),
            },
            FlowEdgeDefinition {
                source: "and_gate".to_string(),
                target: "quality_sink".to_string(),
                source_handle: "output-0".to_string(),
                target_handle: "input-0".to_string(),
            },
        ],
    }
}

/// Creates a list of qualities for complex flow tests.
#[allow(dead_code)]
pub fn create_complex_qualities() -> Vec<Quality> {
    vec![
        Quality {
            name: "Premium".to_string(),
            priority: 1,
        },
        Quality {
            name: "Standard".to_string(),
            priority: 2,
        },
    ]
}

/// Creates sample static data for testing.
#[allow(dead_code)]
pub fn create_sample_static_data() -> HashMap<String, f64> {
    let mut data = HashMap::new();
    data.insert("Temperature".to_string(), 32.5);
    data
}

/// Creates sample dynamic data for testing.
#[allow(dead_code)]
pub fn create_sample_dynamic_data() -> HashMap<String, Vec<HashMap<String, f64>>> {
    let mut data = HashMap::new();
    let mut hole_events = Vec::new();
    let mut hole1 = HashMap::new();
    hole1.insert("Diameter".to_string(), 8.7);
    hole_events.push(hole1);
    data.insert("hole".to_string(), hole_events);
    data
}
