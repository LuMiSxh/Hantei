use crate::ast::{Expression, InputId, InputSource, Value};
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::compiler::CompilationArtifacts;
use crate::error::{BackendError, EvaluationError};
use crate::recipe::{CompiledPathInterpreter, CompiledRecipe};
use crate::trace::TraceFormatter;
use ahash::AHashMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

mod engine;

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
        artifacts: Vec<CompilationArtifacts>,
    ) -> Result<CompiledRecipe, BackendError> {
        let interpreter_paths = artifacts
            .into_iter()
            .map(|artifact| {
                let mut visited = HashMap::new();
                let linked_ast = link_ast(&artifact.ast, &artifact.definitions, &mut visited)?;
                Ok(CompiledPathInterpreter {
                    priority: artifact.priority,
                    name: artifact.name,
                    ast: linked_ast,
                    static_map: artifact.static_map,
                    dynamic_map: artifact.dynamic_map,
                })
            })
            .collect::<Result<_, BackendError>>()?;

        // Return a CompiledRecipe, which can contain interpreter paths.
        Ok(CompiledRecipe::new(Some(interpreter_paths), None))
    }

    fn load(
        &self,
        recipe: crate::recipe::CompiledRecipe,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        let paths = recipe.interpreter_paths.ok_or_else(|| {
            BackendError::InvalidLogic(
                "Recipe file does not contain interpreter artifacts".to_string(),
            )
        })?;
        let executable_paths = paths
            .into_iter()
            .map(|p| (p.priority, p.name, p.ast, p.static_map, p.dynamic_map))
            .collect();
        Ok(Box::new(AstExecutable {
            paths: executable_paths,
        }))
    }
}

struct AstExecutable {
    paths: Vec<(
        i32,
        String,
        Expression,
        AHashMap<String, InputId>,
        AHashMap<String, InputId>,
    )>,
}

fn is_purely_static(expr: &Expression) -> bool {
    match expr {
        Expression::Input(InputSource::Dynamic { .. }) => false,
        Expression::Sum(l, r)
        | Expression::Subtract(l, r)
        | Expression::Multiply(l, r)
        | Expression::Divide(l, r)
        | Expression::And(l, r)
        | Expression::Or(l, r)
        | Expression::Xor(l, r)
        | Expression::Equal(l, r)
        | Expression::NotEqual(l, r)
        | Expression::GreaterThan(l, r)
        | Expression::GreaterThanOrEqual(l, r)
        | Expression::SmallerThan(l, r)
        | Expression::SmallerThanOrEqual(l, r) => is_purely_static(l) && is_purely_static(r),
        Expression::Not(v) | Expression::Abs(v) => is_purely_static(v),
        _ => true,
    }
}

impl ExecutableRecipe for AstExecutable {
    fn evaluate(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        let maybe_result =
            self.paths
                .par_iter()
                .find_map_any(|(priority, name, ast, static_map, dynamic_map)| {
                    let static_vec = match prepare_static_data(static_map, static_data) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let static_rev_map: AHashMap<InputId, String> =
                        static_map.iter().map(|(k, v)| (*v, k.clone())).collect();
                    let dynamic_rev_map: AHashMap<InputId, String> =
                        dynamic_map.iter().map(|(k, v)| (*v, k.clone())).collect();

                    if let Expression::And(l, r) = ast {
                        for branch in [l.as_ref(), r.as_ref()] {
                            if is_purely_static(branch) {
                                let empty_dynamic_vec = Vec::new();
                                let engine = engine::AstEngine::new(
                                    branch,
                                    &static_vec,
                                    &empty_dynamic_vec,
                                    &static_rev_map,
                                    &dynamic_rev_map,
                                );
                                match engine.evaluate() {
                                    Ok(trace)
                                        if matches!(trace.get_outcome(), Value::Bool(false)) =>
                                    {
                                        return None;
                                    }
                                    Err(e) => return Some(Err(e)),
                                    _ => {}
                                }
                            }
                        }
                    }

                    let dynamic_combinations = generate_dynamic_contexts(dynamic_map, dynamic_data);
                    if dynamic_combinations.is_empty() && !dynamic_map.is_empty() {
                        return None;
                    }

                    for context_map in &dynamic_combinations {
                        let dynamic_vec = prepare_dynamic_context(dynamic_map, context_map);
                        let engine = engine::AstEngine::new(
                            ast,
                            &static_vec,
                            &dynamic_vec,
                            &static_rev_map,
                            &dynamic_rev_map,
                        );
                        match engine.evaluate() {
                            Ok(trace) if matches!(trace.get_outcome(), Value::Bool(true)) => {
                                let reason = TraceFormatter::format_trace(&trace);
                                return Some(Ok(EvaluationResult {
                                    quality_name: Some(name.clone()),
                                    quality_priority: Some(*priority),
                                    reason,
                                }));
                            }
                            Err(e) => return Some(Err(e)),
                            _ => {} // Continue to next combination
                        }
                    }
                    None // No combination triggered this quality path
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

fn prepare_static_data(
    map: &AHashMap<String, InputId>,
    data: &AHashMap<String, f64>,
) -> Result<Vec<Value>, EvaluationError> {
    let mut vec = vec![Value::Null; map.len()];
    for (name, &id) in map {
        let value = data
            .get(name)
            .map(|v| Value::Number(*v))
            .ok_or_else(|| EvaluationError::InputNotFound(name.clone()))?;
        if id as usize >= vec.len() {
            vec.resize((id + 1) as usize, Value::Null);
        }
        vec[id as usize] = value;
    }
    Ok(vec)
}

fn prepare_dynamic_context(
    map: &AHashMap<String, InputId>,
    context: &AHashMap<String, &AHashMap<String, f64>>,
) -> Vec<Value> {
    let mut vec = vec![Value::Null; map.len()];
    for (key, &id) in map {
        let (event_name, field_name) = key.split_once('.').unwrap();
        if let Some(instance) = context.get(event_name) {
            if let Some(value) = instance.get(field_name) {
                if id as usize >= vec.len() {
                    vec.resize((id + 1) as usize, Value::Null);
                }
                vec[id as usize] = Value::Number(*value);
            }
        }
    }
    vec
}

fn generate_dynamic_contexts<'a>(
    dynamic_map: &AHashMap<String, InputId>,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
) -> Vec<AHashMap<String, &'a AHashMap<String, f64>>> {
    let required_events: HashSet<&str> = dynamic_map
        .keys()
        .map(|k| k.split_once('.').unwrap().0)
        .collect();
    if required_events.is_empty() {
        return vec![AHashMap::new()];
    }

    let mut event_types: Vec<_> = required_events.into_iter().collect();
    event_types.sort_by_key(|event_type| dynamic_data.get(*event_type).map_or(0, |v| v.len()));

    let mut combinations = vec![AHashMap::new()];
    for event_type in event_types {
        let instances = match dynamic_data.get(event_type) {
            Some(inst) if !inst.is_empty() => inst,
            _ => {
                return vec![];
            }
        };
        let mut next_combinations = Vec::new();
        for combo in &combinations {
            for instance in instances {
                let mut new_combo = combo.clone();
                new_combo.insert(event_type.to_string(), instance);
                next_combinations.push(new_combo);
            }
        }
        combinations = next_combinations;
    }
    combinations
}

fn link_ast(
    expr: &Expression,
    definitions: &AHashMap<u64, Expression>,
    visited: &mut HashMap<u64, Expression>,
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
