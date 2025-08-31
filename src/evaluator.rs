use crate::backend::{BackendChoice, EvaluationBackend, ExecutableRecipe};
use crate::compiler::CompilationArtifacts;
use crate::error::{BackendError, EvaluationError};
pub use crate::interpreter::EvaluationResult;
use crate::recipe::CompiledRecipe;
use ahash::AHashMap;

/// The main entry point for evaluating compiled recipes against data.
///
/// An `Evaluator` is created by choosing a backend and providing the compiled ASTs.
/// It can be used repeatedly and safely across multiple threads.
pub struct Evaluator {
    executable: Box<dyn ExecutableRecipe>,
}

impl Evaluator {
    /// Creates a new evaluator by compiling the compilation artifacts with the chosen backend.
    pub fn new(
        choice: BackendChoice,
        artifacts: Vec<CompilationArtifacts>,
    ) -> Result<Self, BackendError> {
        let backend: Box<dyn EvaluationBackend> = match choice {
            BackendChoice::Interpreter => Box::new(crate::interpreter::InterpreterBackend),
            BackendChoice::Bytecode => Box::new(crate::bytecode::BytecodeBackend),
        };

        let compiled_recipe = backend.compile(artifacts)?;
        let executable = backend.load(compiled_recipe)?;

        Ok(Self { executable })
    }
    /// Creates a new evaluator from a compiled recipe loaded from a file.
    pub fn from_file(choice: BackendChoice, path: &str) -> Result<Self, BackendError> {
        let recipe = CompiledRecipe::from_file(path)?;
        Self::from_compiled_recipe(choice, recipe)
    }

    /// Creates a new evaluator from a compiled recipe provided as bytes.
    pub fn from_bytes(choice: BackendChoice, bytes: &[u8]) -> Result<Self, BackendError> {
        let recipe = CompiledRecipe::from_bytes(bytes)?;
        Self::from_compiled_recipe(choice, recipe)
    }

    /// Internal helper to create an evaluator from a compiled recipe.
    fn from_compiled_recipe(
        choice: BackendChoice,
        recipe: CompiledRecipe,
    ) -> Result<Self, BackendError> {
        let backend: Box<dyn EvaluationBackend> = match choice {
            BackendChoice::Interpreter => Box::new(crate::interpreter::InterpreterBackend),
            BackendChoice::Bytecode => Box::new(crate::bytecode::BytecodeBackend),
        };
        let executable = backend.load(recipe)?;
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
