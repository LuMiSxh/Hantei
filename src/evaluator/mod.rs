use crate::ast::{Expression, Value};
use crate::error::EvaluationError;
use crate::trace::TraceFormatter;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

mod dynamic;
mod engine;

use dynamic::DynamicEvaluator;
use engine::AstEngine;

/// The result of an evaluation run.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The name of the highest-priority quality that was triggered.
    /// `None` if no quality path evaluated to `true`.
    pub quality_name: Option<String>,
    /// The priority of the triggered quality. `None` if no quality was triggered.
    pub quality_priority: Option<i32>,
    /// A human-readable explanation of the logic that led to the result.
    pub reason: String,
}

/// Evaluates compiled Abstract Syntax Trees (ASTs) against runtime data.
///
/// An `Evaluator` is created from the output of a `Compiler`. It can be used
/// repeatedly and safely across multiple threads to evaluate different data sets.
pub struct Evaluator {
    /// The compiled and optimized quality paths, sorted by priority.
    pub quality_paths: Vec<(i32, String, Expression)>,
}

impl Evaluator {
    /// Creates a new evaluator with a set of compiled quality paths.
    /// The paths will be automatically sorted by priority (ascending).
    pub fn new(mut quality_paths: Vec<(i32, String, Expression)>) -> Self {
        quality_paths.sort_by_key(|(priority, _, _)| *priority);
        Self { quality_paths }
    }

    /// Evaluates all quality paths in parallel against the provided data.
    ///
    /// This method is the core of the runtime engine. It efficiently finds the first,
    /// highest-priority quality path that evaluates to `true` for the given data.
    ///
    /// # Arguments
    ///
    /// * `static_data`: A `HashMap` of static measurements, where keys are measurement
    ///   names (e.g., `"Humidity"`) and values are numbers.
    /// * `dynamic_data`: A `HashMap` representing event-based data. Keys are event
    ///   types (e.g., `"hole"`), and values are a `Vec` of `HashMap`s, where each
    ///   inner map is a distinct instance of that event.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(EvaluationResult)`: On successful evaluation. The `quality_name` inside will be `None` if no path was triggered.
    /// * `Err(EvaluationError)`: If a fatal error occurred, such as a type mismatch or a missing input value.
    pub fn eval(
        &self,
        static_data: &HashMap<String, f64>,
        dynamic_data: &HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        let maybe_result = self
            .quality_paths
            .par_iter()
            .find_map_any(|(priority, name, ast)| {
                let mut required_events = HashSet::new();
                ast.get_required_events(&mut required_events);

                let eval_result = if required_events.is_empty() {
                    let empty_context = HashMap::new();
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
