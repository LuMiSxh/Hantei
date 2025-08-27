/// The complete, canonical definition of a logic flow, ready for compilation.
/// This is the target structure for any custom data model conversion.
#[derive(Debug, Clone, Default)]
pub struct FlowDefinition {
    pub nodes: Vec<FlowNodeDefinition>,
    pub edges: Vec<FlowEdgeDefinition>,
}

/// Defines a single node (an operation or data source) in the logic flow.
#[derive(Debug, Clone)]
pub struct FlowNodeDefinition {
    pub id: String,
    pub operation_type: String,
    pub input_type: Option<String>,
    pub literal_values: Option<Vec<serde_json::Value>>,
    pub data_fields: Option<Vec<DataFieldDefinition>>,
}

/// Defines a data field that a node can output (previously a "case").
#[derive(Debug, Clone)]
pub struct DataFieldDefinition {
    pub id: u32,
    pub name: String,
    pub data_type: Option<String>,
}

/// Defines a connection between two nodes in the logic flow.
#[derive(Debug, Clone)]
pub struct FlowEdgeDefinition {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}
