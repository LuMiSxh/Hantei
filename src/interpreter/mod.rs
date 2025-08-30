use crate::ast::{Expression, Value};
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::error::{BackendError, EvaluationError};
use crate::trace::TraceFormatter;
use ahash::AHashMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

mod dynamic;
mod engine;

use dynamic::DynamicEvaluator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    pub quality_name: Option<String>,
    pub quality_priority: Option<i32>,
    pub reason: String,
}

pub struct InterpreterBackend;

impl EvaluationBackend for InterpreterBackend {
    fn compile(
        &self,
        paths: Vec<(i32, String, Expression, AHashMap<u64, Expression>)>,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        // The "linking" phase: transform the AST graph into a simple tree for each path.
        let linked_paths = paths
            .into_iter()
            .map(|(p, n, ast, defs)| {
                // We use a temporary HashMap for memoization *during this link pass only* to avoid re-linking the same sub-tree.
                let mut visited = HashMap::new();
                let linked_ast = link_ast(&ast, &defs, &mut visited)?;
                Ok((p, n, linked_ast))
            })
            .collect::<Result<_, BackendError>>()?;

        Ok(Box::new(AstExecutable {
            paths: linked_paths,
        }))
    }
}

/// The executable now holds fully self-contained, "linked" ASTs with no References.
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
            // We can now use the simpler get_required_events because the AST is just a tree.
            ast.get_required_events(&mut required_events);

            let eval_result = if required_events.is_empty() {
                let empty_context = AHashMap::new();
                let engine = engine::AstEngine::new(ast, static_data, &empty_context);
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

/// Recursively walks an expression, inlining any `Reference` nodes to produce a flat tree.
fn link_ast(
    expr: &Expression,
    definitions: &AHashMap<u64, Expression>,
    visited: &mut HashMap<u64, Expression>, // Memoization for this link pass
) -> Result<Expression, BackendError> {
    match expr {
        Expression::Reference(id) => {
            if let Some(cached) = visited.get(id) {
                return Ok(cached.clone());
            }
            let def = definitions.get(id).ok_or_else(|| {
                BackendError::InvalidLogic(format!(
                    "CSE Reference ID #{} not found during linking",
                    id
                ))
            })?;
            let linked_def = link_ast(def, definitions, visited)?;
            visited.insert(*id, linked_def.clone());
            Ok(linked_def)
        }
        // --- Nodes with Children ---
        Expression::Sum(l, r) => Ok(Expression::Sum(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Subtract(l, r) => Ok(Expression::Subtract(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Multiply(l, r) => Ok(Expression::Multiply(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Divide(l, r) => Ok(Expression::Divide(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Abs(v) => Ok(Expression::Abs(Box::new(link_ast(
            v,
            definitions,
            visited,
        )?))),
        Expression::Not(v) => Ok(Expression::Not(Box::new(link_ast(
            v,
            definitions,
            visited,
        )?))),
        Expression::And(l, r) => Ok(Expression::And(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Or(l, r) => Ok(Expression::Or(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Xor(l, r) => Ok(Expression::Xor(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::Equal(l, r) => Ok(Expression::Equal(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::NotEqual(l, r) => Ok(Expression::NotEqual(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::GreaterThan(l, r) => Ok(Expression::GreaterThan(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::GreaterThanOrEqual(l, r) => Ok(Expression::GreaterThanOrEqual(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::SmallerThan(l, r) => Ok(Expression::SmallerThan(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),
        Expression::SmallerThanOrEqual(l, r) => Ok(Expression::SmallerThanOrEqual(
            Box::new(link_ast(l, definitions, visited)?),
            Box::new(link_ast(r, definitions, visited)?),
        )),

        // --- Leaf Nodes (no children to link) ---
        Expression::Literal(_) | Expression::Input(_) => Ok(expr.clone()),
    }
}
