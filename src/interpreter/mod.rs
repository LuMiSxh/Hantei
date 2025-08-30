use crate::ast::{Expression, Value};
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::error::{BackendError, EvaluationError};
use crate::trace::TraceFormatter;
use ahash::AHashMap;
use rayon::prelude::*;
use std::collections::HashSet;

mod dynamic;
mod engine;

use dynamic::DynamicEvaluator;
use engine::AstEngine;

/// The result of an evaluation run.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    /// The name of the highest-priority quality that was triggered.
    /// `None` if no quality path evaluated to `true`.
    pub quality_name: Option<String>,
    /// The priority of the triggered quality. `None` if no quality was triggered.
    pub quality_priority: Option<i32>,
    /// A human-readable explanation of the logic that led to the result.
    pub reason: String,
}

/// A backend that directly interprets the AST at runtime.
pub struct InterpreterBackend;

impl EvaluationBackend for InterpreterBackend {
    fn compile(
        &self,
        paths: Vec<(i32, String, Expression)>,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        // The "compilation" for the interpreter is a no-op; it just stores the paths.
        Ok(Box::new(AstExecutable { paths }))
    }
}

/// The `ExecutableRecipe` for the interpreter backend. It holds the ASTs to be walked.
struct AstExecutable {
    paths: Vec<(i32, String, Expression)>,
}

impl ExecutableRecipe for AstExecutable {
    fn evaluate(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        let maybe_result = self.paths.par_iter().find_map_any(|(priority, name, ast)| {
            let mut required_events = HashSet::new();
            ast.get_required_events(&mut required_events);

            let eval_result = if required_events.is_empty() {
                let empty_context = AHashMap::new();
                let engine = AstEngine::new(ast, static_data, &empty_context);
                engine.evaluate().map(Some)
            } else {
                let dynamic_eval =
                    DynamicEvaluator::new(ast, &required_events, static_data, dynamic_data);
                dynamic_eval.evaluate()
            };

            match eval_result {
                Ok(Some(trace)) if matches!(trace.get_outcome(), Value::Bool(true)) => {
                    Some(Ok(EvaluationResult {
                        quality_name: Some(name.clone()),
                        quality_priority: Some(*priority),
                        reason: TraceFormatter::format_trace(&trace),
                    }))
                }
                Err(e) => Some(Err(e)),
                _ => None,
            }
        });

        match maybe_result {
            Some(Ok(result)) => Ok(result),
            Some(Err(e)) => Err(e),
            None => Ok(EvaluationResult {
                quality_name: None,
                quality_priority: None,
                reason: "No quality triggered".to_string(),
            }),
        }
    }
}
