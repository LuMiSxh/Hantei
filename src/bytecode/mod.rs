pub mod compiler;
pub mod opcode;
pub mod visualizer;
pub mod vm;

use crate::ast::Value;
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::compiler::CompilationArtifacts;
use crate::error::{BackendError, EvaluationError};
use crate::interpreter::EvaluationResult;
use crate::recipe::{CompiledPathBytecode, CompiledRecipe};
use ahash::AHashMap;
use compiler::BytecodeProgram;
use itertools::Itertools;
use std::collections::HashSet;
use vm::Vm;

pub struct BytecodeBackend;

impl EvaluationBackend for BytecodeBackend {
    fn compile(
        &self,
        artifacts: Vec<CompilationArtifacts>,
    ) -> Result<CompiledRecipe, BackendError> {
        let bytecode_programs = artifacts
            .into_iter()
            .map(|a| {
                // We have to get rid of the ast here because we no longer need it.
                let program = compiler::compile_to_program(
                    &a.ast,
                    &a.definitions,
                    &a.static_map,
                    &a.dynamic_map,
                )?;

                Ok(CompiledPathBytecode {
                    priority: a.priority,
                    name: a.name,
                    program,
                })
            })
            .collect::<Result<Vec<_>, BackendError>>()?;

        Ok(CompiledRecipe::new(None, Some(bytecode_programs)))
    }

    fn load(&self, recipe: CompiledRecipe) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        let programs = recipe.bytecode_programs.ok_or_else(|| {
            BackendError::InvalidLogic(
                "Recipe file does not contain bytecode artifacts".to_string(),
            )
        })?;

        let compiled_artifacts = programs
            .into_iter()
            .map(|p| (p.priority, p.name, p.program))
            .collect();

        Ok(Box::new(BytecodeExecutable { compiled_artifacts }))
    }
}

struct BytecodeExecutable {
    compiled_artifacts: Vec<(i32, String, BytecodeProgram)>,
}

impl ExecutableRecipe for BytecodeExecutable {
    fn evaluate(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        let prepared_static_data = prepare_all_static_data(&self.compiled_artifacts, static_data)?;

        let maybe_result = self.compiled_artifacts.iter().enumerate().find_map(
            |(prog_idx, (priority, name, program))| {
                let (event_names, event_instances) = prepare_dynamic_events(program, dynamic_data);

                if event_instances.iter().any(|v| v.is_empty()) {
                    return None; // If any required event type has no instances, we can't match.
                }

                // If there are no dynamic events required, we still need one empty context to run against.
                let combinations_iterator: Box<dyn Iterator<Item = Vec<&AHashMap<String, f64>>>> =
                    if event_instances.is_empty() {
                        Box::new(std::iter::once(Vec::new()))
                    } else {
                        Box::new(event_instances.into_iter().multi_cartesian_product())
                    };

                let static_vec = &prepared_static_data[prog_idx];

                for combination in combinations_iterator {
                    // Build the context map for this single combination
                    let context_map: AHashMap<&str, _> = event_names
                        .iter()
                        .map(|s| s.as_str())
                        .zip(combination.into_iter())
                        .collect();

                    let dynamic_vec = prepare_dynamic_context(program, &context_map);
                    let mut vm = Vm::new(program, static_vec, &dynamic_vec);
                    match vm.run() {
                        Ok(Value::Bool(true)) => {
                            return Some(Ok(EvaluationResult {
                                quality_name: Some(name.clone()),
                                quality_priority: Some(*priority),
                                reason: format!("Bytecode evaluation for '{}' returned true", name),
                            }));
                        }
                        Ok(_) => continue,
                        Err(e) => return Some(Err(EvaluationError::BackendError(e.to_string()))),
                    }
                }
                None
            },
        );

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

fn prepare_all_static_data(
    artifacts: &[(i32, String, BytecodeProgram)],
    static_data: &AHashMap<String, f64>,
) -> Result<Vec<Vec<Value>>, EvaluationError> {
    artifacts
        .iter()
        .map(|(_, _, program)| {
            let mut static_vec = vec![Value::Null; program.static_map.len()];
            for (name, &id) in &program.static_map {
                let value = static_data
                    .get(name)
                    .map(|v| Value::Number(*v))
                    .ok_or_else(|| EvaluationError::InputNotFound(name.clone()))?;
                static_vec[id as usize] = value;
            }
            Ok(static_vec)
        })
        .collect()
}

fn prepare_dynamic_context(
    program: &BytecodeProgram,
    context: &AHashMap<&str, &AHashMap<String, f64>>,
) -> Vec<Value> {
    let mut dynamic_vec = vec![Value::Null; program.dynamic_map.len()];
    for (key, &id) in &program.dynamic_map {
        let (event_name, field_name) = key.split_once('.').unwrap();
        if let Some(instance) = context.get(event_name) {
            if let Some(value) = instance.get(field_name) {
                dynamic_vec[id as usize] = Value::Number(*value);
            }
        }
    }
    dynamic_vec
}

fn prepare_dynamic_events<'a>(
    program: &BytecodeProgram,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
) -> (Vec<String>, Vec<Vec<&'a AHashMap<String, f64>>>) {
    let mut required_events = HashSet::new();
    for key in program.dynamic_map.keys() {
        required_events.insert(key.split_once('.').unwrap().0);
    }
    if required_events.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let event_names: Vec<String> = required_events.into_iter().map(|s| s.to_string()).collect();
    let mut event_instances = Vec::with_capacity(event_names.len());

    for event_name in &event_names {
        match dynamic_data.get(event_name) {
            Some(instances) => {
                event_instances.push(instances.iter().collect());
            }
            None => {
                event_instances.push(Vec::new());
            }
        }
    }
    (event_names, event_instances)
}
