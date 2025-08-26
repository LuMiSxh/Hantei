use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CompileError {
    #[error("Node '{0}' not found in recipe")]
    NodeNotFound(String),
    #[error("Failed to parse node input reference '{0}'")]
    InvalidInputRef(String),
    #[error("Node '{0}' has a missing or invalid operation type")]
    InvalidNodeType(String),
    #[error("Node '{0}' is connected to a quality input, but was not found")]
    QualityTriggerNodeNotFound(String),
    #[error("JSON parsing error: {0}")]
    JsonParseError(String),
}

#[derive(Error, Debug, Clone)]
pub enum EvaluationError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
    #[error("Input source '{0}' not found in provided data")]
    InputNotFound(String),
}
