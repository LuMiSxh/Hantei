pub mod compiler;
pub mod opcode;
pub mod visualizer;
pub mod vm;

use crate::ast::{Expression, Value};
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::bytecode::opcode::OpCode;
use crate::error::{BackendError, EvaluationError};
use crate::interpreter::EvaluationResult;
use ahash::AHashMap;
use compiler::BytecodeProgram;
use rayon::prelude::*;
use std::collections::HashSet;
use vm::Vm;

/// A backend that compiles ASTs to register-based bytecode and runs them on a VM.
pub struct BytecodeBackend;

impl EvaluationBackend for BytecodeBackend {
    fn compile(
        &self,
        paths: Vec<(i32, String, Expression, AHashMap<u64, Expression>)>,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        let compiled_programs = paths
            .into_par_iter()
            .map(|(p, n, ast, definitions)| {
                compiler::compile_to_program(&ast, &definitions).map(|program| (p, n, program))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Box::new(BytecodeExecutable { compiled_programs }))
    }
}

struct BytecodeExecutable {
    compiled_programs: Vec<(i32, String, BytecodeProgram)>,
}

impl ExecutableRecipe for BytecodeExecutable {
    fn evaluate(
        &self,
        static_data: &AHashMap<String, f64>,
        dynamic_data: &AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Result<EvaluationResult, EvaluationError> {
        let maybe_result =
            self.compiled_programs
                .par_iter()
                .find_map_any(|(priority, name, program)| {
                    // The evaluation logic is now much simpler. We just run the VM for each combination.
                    let dynamic_combinations = generate_dynamic_contexts(program, dynamic_data);

                    for context in &dynamic_combinations {
                        let mut vm = Vm::new(program, static_data, context);
                        match vm.run() {
                            Ok(Value::Bool(true)) => {
                                return Some(Ok(EvaluationResult {
                                    quality_name: Some(name.clone()),
                                    quality_priority: Some(*priority),
                                    reason: format!(
                                        "Bytecode evaluation for '{}' returned true",
                                        name
                                    ),
                                }));
                            }
                            Ok(_) => continue, // Continue to next combination
                            Err(e) => {
                                return Some(Err(EvaluationError::BackendError(e.to_string())));
                            }
                        }
                    }
                    None // No combination triggered this quality
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

// Generates all combinations of dynamic contexts required by the program.
fn generate_dynamic_contexts<'a>(
    program: &BytecodeProgram,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
) -> Vec<AHashMap<String, &'a AHashMap<String, f64>>> {
    let mut required_events = HashSet::new();
    // Scan main and subroutines for dynamic load instructions
    let all_opcodes = program
        .main
        .iter()
        .chain(program.subroutines.values().flatten());
    for op in all_opcodes {
        if let OpCode::LoadDynamic(_, event, _) = op {
            required_events.insert(event.as_str());
        }
    }

    if required_events.is_empty() {
        return vec![AHashMap::new()]; // Return one context for a single run
    }

    let mut event_types: Vec<_> = required_events.into_iter().collect();
    event_types.sort_by_key(|event_type| dynamic_data.get(*event_type).map_or(0, |v| v.len()));

    let mut combinations = vec![AHashMap::new()];
    for event_type in event_types {
        let instances = match dynamic_data.get(event_type) {
            Some(inst) if !inst.is_empty() => inst,
            _ => {
                // If a required event has no instances, no combination is possible.
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
