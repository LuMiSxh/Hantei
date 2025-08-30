pub mod compiler;
pub mod opcode;
pub mod visualizer;
pub mod vm;

use crate::ast::{Expression, Value};
use crate::backend::{EvaluationBackend, ExecutableRecipe};
use crate::error::{BackendError, EvaluationError, VmError};
use crate::interpreter::EvaluationResult;
use ahash::AHashMap;
use compiler::BytecodeProgram;
use rayon::prelude::*;
use std::collections::HashSet;
use vm::Vm;

/// A backend that compiles ASTs to bytecode and runs them on a VM.
pub struct BytecodeBackend;

impl EvaluationBackend for BytecodeBackend {
    fn compile(
        &self,
        paths: Vec<(i32, String, Expression, AHashMap<u64, Expression>)>,
    ) -> Result<Box<dyn ExecutableRecipe>, BackendError> {
        let compiled_programs = paths
            .into_iter()
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
                    let mut required_events = HashSet::new();
                    // Scan main program for dynamic loads
                    for op in &program.main {
                        if let opcode::OpCode::LoadDynamic(event, _) = op {
                            required_events.insert(event.clone());
                        }
                    }
                    // Scan all subroutines as well
                    for subroutine in program.subroutines.values() {
                        for op in subroutine {
                            if let opcode::OpCode::LoadDynamic(event, _) = op {
                                required_events.insert(event.clone());
                            }
                        }
                    }

                    let vm_result = if required_events.is_empty() {
                        let dynamic_context = AHashMap::new();
                        let mut vm = Vm::new(program, static_data, &dynamic_context);
                        vm.run().map(Some)
                    } else {
                        let dynamic_eval = DynamicVmEvaluator::new(
                            program,
                            &required_events,
                            static_data,
                            dynamic_data,
                        );
                        dynamic_eval.evaluate()
                    };

                    match vm_result {
                        Ok(Some(Value::Bool(true))) => Some(Ok(EvaluationResult {
                            quality_name: Some(name.clone()),
                            quality_priority: Some(*priority),
                            reason: format!("Bytecode evaluation for '{}' returned true", name),
                        })),
                        Err(e) => Some(Err(EvaluationError::BackendError(e.to_string()))),
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

// --- Dynamic Evaluator for the VM ---

struct DynamicVmEvaluator<'a> {
    program: &'a BytecodeProgram,
    event_types: Vec<String>,
    static_data: &'a AHashMap<String, f64>,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
}

impl<'a> DynamicVmEvaluator<'a> {
    fn new(
        program: &'a BytecodeProgram,
        required_events: &HashSet<String>,
        static_data: &'a AHashMap<String, f64>,
        dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Self {
        let mut event_types: Vec<String> = required_events.iter().cloned().collect();
        event_types.sort_by_key(|event_type| dynamic_data.get(event_type).map_or(0, |v| v.len()));
        Self {
            program,
            event_types,
            static_data,
            dynamic_data,
        }
    }

    fn evaluate(&self) -> Result<Option<Value>, VmError> {
        let mut context = AHashMap::new();
        self.evaluate_recursive(&self.event_types, &mut context)
    }

    fn evaluate_recursive(
        &self,
        remaining_events: &[String],
        context: &mut AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Result<Option<Value>, VmError> {
        if remaining_events.is_empty() {
            let mut vm = Vm::new(self.program, self.static_data, context);
            let result = vm.run()?;
            if matches!(result, Value::Bool(true)) {
                return Ok(Some(result));
            }
            return Ok(None);
        }

        let current_event_type = &remaining_events[0];
        let next_event_types = &remaining_events[1..];

        if let Some(instances) = self.dynamic_data.get(current_event_type) {
            if instances.is_empty() {
                return Ok(None);
            }
            for instance in instances {
                context.insert(current_event_type.clone(), instance);
                if let Some(result) = self.evaluate_recursive(next_event_types, context)? {
                    return Ok(Some(result));
                }
            }
        } else {
            return Ok(None);
        }

        Ok(None)
    }
}
