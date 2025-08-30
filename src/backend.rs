use crate::error::{BackendError, EvaluationError};
use crate::interpreter::EvaluationResult;
use crate::prelude::*;
use ahash::AHashMap;

/// A compiled, runnable recipe that can be evaluated against data.
/// This is the "artifact" produced by a backend's compile step.
pub trait ExecutableRecipe: Send + Sync {
    fn evaluate(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError>;
}

/// A trait for an evaluation backend that transforms ASTs into an `ExecutableRecipe`.
pub trait EvaluationBackend {
    /// Compiles the high-level AST paths into a backend-specific executable.
    fn compile(
        &self,
        paths: Vec<(i32, String, Expression)>,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError>;
}

/// The available backends for evaluation.
#[derive(Debug, Clone, Copy)]
pub enum BackendChoice {
    /// Directly walks the AST. Good for debugging, but slower.
    Interpreter,
    /// Compiles to custom bytecode and runs it on a stack-based VM. Faster.
    Bytecode,
}
