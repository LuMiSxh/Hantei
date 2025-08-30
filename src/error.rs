use crate::{ast::Value, bytecode::opcode::OpCode};
use thiserror::Error;

/// Errors that can occur during the recipe compilation phase (parsing into an AST).
#[derive(Error, Debug, Clone)]
pub enum AstBuildError {
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

/// Errors that can occur when a backend compiles an AST into an executable format.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BackendError {
    #[error("Unsupported AST node for this backend: {0}")]
    UnsupportedAstNode(String),

    #[error("Backend resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Invalid logic encountered during backend compilation: {0}")]
    InvalidLogic(String),

    #[error("An unexpected backend error occurred: {0}")]
    Generic(String),
}

/// Errors that can occur during the AST evaluation phase (Interpreter).
#[derive(Error, Debug, Clone, PartialEq)]
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

    #[error("A backend evaluation error occurred: {0}")]
    BackendError(String),
}

/// Errors that can occur during the Bytecode VM execution.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum VmError {
    #[error("Stack underflow: expected a value on the stack, but it was empty")]
    StackUnderflow,

    #[error("Type mismatch in VM: expected {expected}, but found value '{found}'")]
    TypeMismatch { expected: String, found: Value },

    #[error("Invalid instruction pointer address: {0}")]
    InvalidIp(usize),

    #[error("Unhandled OpCode encountered: {0:?}")]
    UnhandledOpCode(OpCode),

    #[error("Invalid subroutine ID: {0}")]
    UnknownSubroutine(u64),

    #[error("Input source '{0}' not found in the provided data context")]
    InputNotFound(String),
}

/// Errors that can occur when converting a custom user format into a Hantei `FlowDefinition`.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RecipeConversionError {
    #[error("Invalid custom data format: {0}")]
    ValidationError(String),
}
