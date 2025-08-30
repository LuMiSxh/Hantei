use crate::backend::{BackendChoice, EvaluationBackend, ExecutableRecipe};
use crate::error::{BackendError, EvaluationError};
pub use crate::interpreter::EvaluationResult;
use crate::prelude::*;
use ahash::AHashMap;

/// The main entry point for evaluating compiled recipes against data.
///
/// An `Evaluator` is created by choosing a backend and providing the compiled ASTs.
/// It can be used repeatedly and safely across multiple threads.
pub struct Evaluator {
    executable: Box<dyn ExecutableRecipe>,
}

impl Evaluator {
    /// Creates a new evaluator by compiling the AST paths with the chosen backend.
    pub fn new(
        choice: BackendChoice,
        paths: Vec<(i32, String, Expression, AHashMap<u64, Expression>)>,
    ) -> Result<Self, BackendError> {
        let backend: Box<dyn EvaluationBackend> = match choice {
            BackendChoice::Interpreter => Box::new(crate::interpreter::InterpreterBackend),
            BackendChoice::Bytecode => Box::new(crate::bytecode::BytecodeBackend),
        };

        let executable = backend.compile(paths)?;
        Ok(Self { executable })
    }

    /// Evaluates the compiled recipe against the provided data.
    pub fn eval(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        self.executable.evaluate(static_data, dynamic_data)
    }
}
