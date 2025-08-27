use serde::Deserialize;

/// UI node data containing operation type and configuration
#[derive(Debug, Deserialize, Clone)]
pub struct UiNodeData {
    #[serde(alias = "realNodeType")]
    pub real_node_type: String,
    #[serde(alias = "realInputType")]
    pub real_input_type: Option<String>,
    pub values: Option<Vec<serde_json::Value>>,
    pub cases: Option<Vec<UiNodeCase>>,
}

/// Case information for dynamic nodes
#[derive(Debug, Deserialize, Clone)]
pub struct UiNodeCase {
    #[serde(alias = "caseId")]
    pub case_id: u32,
    #[serde(alias = "caseName")]
    pub case_name: String,
    #[serde(default)]
    #[serde(alias = "realCaseType")]
    pub real_case_type: Option<String>,
}

/// UI node wrapper structure
#[derive(Debug, Deserialize)]
pub struct UiNodeWrapper {
    #[serde(alias = "nodeData")]
    pub node_data: UiNodeData,
}

/// UI node with ID and data
#[derive(Debug, Deserialize)]
pub struct UiNode {
    pub id: String,
    pub data: UiNodeWrapper,
}

/// UI edge connecting nodes
#[derive(Debug, Deserialize)]
pub struct UiEdge {
    pub source: String,
    #[serde(alias = "sourceHandle")]
    pub source_handle: String,
    pub target: String,
    #[serde(alias = "targetHandle")]
    pub target_handle: String,
}

/// Complete UI recipe structure
#[derive(Debug, Deserialize)]
pub struct UiRecipe {
    pub nodes: Vec<UiNode>,
    pub edges: Vec<UiEdge>,
}

/// Quality definition with name and priority
#[derive(Debug, Deserialize, Clone)]
pub struct Quality {
    pub name: String,
    pub priority: i32,
}
