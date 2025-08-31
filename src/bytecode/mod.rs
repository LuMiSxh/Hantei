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
use rayon::prelude::*;
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
                // The AST is passed to the compiler...
                let program = compiler::compile_to_program(
                    &a.ast,
                    &a.definitions,
                    &a.static_map,
                    &a.dynamic_map,
                )?;
                // ...but not stored in the final artifact for bytecode.
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

        let maybe_result = self.compiled_artifacts.par_iter().enumerate().find_map_any(
            |(prog_idx, (priority, name, program))| {
                let dynamic_combinations = generate_dynamic_contexts(program, dynamic_data);

                if dynamic_combinations.is_empty() && !program.dynamic_map.is_empty() {
                    return None;
                }

                let static_vec = &prepared_static_data[prog_idx];
                for context_map in &dynamic_combinations {
                    let dynamic_vec = prepare_dynamic_context(program, context_map);
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
    context: &AHashMap<String, &AHashMap<String, f64>>,
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

fn generate_dynamic_contexts<'a>(
    program: &BytecodeProgram,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
) -> Vec<AHashMap<String, &'a AHashMap<String, f64>>> {
    let mut required_events = HashSet::new();
    for key in program.dynamic_map.keys() {
        required_events.insert(key.split_once('.').unwrap().0);
    }
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
