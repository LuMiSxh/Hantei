use crate::ast::Value;
use thiserror::Error;

/// Errors that can occur during the recipe compilation phase.
#[derive(Error, Debug, Clone)]
pub enum CompileError {
    #[error("Failed to parse recipe JSON: {0}")]
    JsonParseError(String),

    #[error(
        "Node '{missing_node_id}' not found, which is required by a connection from node '{source_node_id}'"
    )]
    NodeNotFound {
        missing_node_id: String,
        source_node_id: String,
    },

    #[error("Node '{node_id}' has an unregistered or invalid operation type: '{type_name}'")]
    InvalidNodeType { node_id: String, type_name: String },

    #[error(
        "A connection to node '{target_node_id}' on handle {target_handle_index} is invalid: {message}"
    )]
    ConnectionError {
        target_node_id: String,
        target_handle_index: u32,
        message: String,
    },

    #[error("Quality trigger node '{0}' is connected, but was not found in the recipe")]
    QualityTriggerNodeNotFound(String),
}

/// Errors that can occur during the AST evaluation phase.
#[derive(Error, Debug, Clone)]
pub enum EvaluationError {
    #[error(
        "Type mismatch during operation '{operation}': expected {expected}, but found value '{found}'"
    )]
    TypeMismatch {
        operation: String,
        expected: String,
        found: Value,
    },

    #[error("Input source '{0}' not found in the provided data context")]
    InputNotFound(String),
}

/// Errors that can occur when converting a custom user format into a Hantei `FlowDefinition`.
#[derive(Error, Debug, Clone)]
pub enum RecipeConversionError {
    #[error("Invalid custom data: {0}")]
    ValidationError(String),
}
