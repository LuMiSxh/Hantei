use crate::ast::{Expression, InputSource};
use crate::bytecode::opcode::{Address, InputId, OpCode, Register};
use crate::error::BackendError;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// State for the register allocator, including a pool for reuse.
#[derive(Default, Debug)]
struct RegisterAllocator {
    next_register: Register,
    free_registers: Vec<Register>,
}

impl RegisterAllocator {
    fn new() -> Self {
        Default::default()
    }

    /// Allocates a new or recycled register.
    fn alloc(&mut self) -> Result<Register, BackendError> {
        if let Some(reg) = self.free_registers.pop() {
            Ok(reg)
        } else {
            let reg = self.next_register;
            self.next_register = self.next_register.checked_add(1).ok_or_else(|| {
                BackendError::ResourceLimitExceeded("Register limit reached".to_string())
            })?;
            Ok(reg)
        }
    }

    /// Returns a register to the pool for reuse.
    fn free(&mut self, reg: Register) {
        // Simple check to avoid double-freeing, which can happen with complex liveness.
        if !self.free_registers.contains(&reg) {
            self.free_registers.push(reg);
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub main: Vec<OpCode>,
    pub subroutines: AHashMap<u64, Vec<OpCode>>,
    pub static_map: AHashMap<String, InputId>,
    pub dynamic_map: AHashMap<String, InputId>,
}

pub struct BytecodeCompiler<'a> {
    definitions: &'a AHashMap<u64, Expression>,
    program: BytecodeProgram,
    compiled_subroutines: AHashMap<u64, ()>,
    allocator: RegisterAllocator,
}

pub fn compile_to_program(
    expr: &Expression,
    definitions: &AHashMap<u64, Expression>,
    static_map: &AHashMap<String, InputId>,
    dynamic_map: &AHashMap<String, InputId>,
) -> Result<BytecodeProgram, BackendError> {
    let mut compiler = BytecodeCompiler {
        definitions,
        program: BytecodeProgram {
            static_map: static_map.clone(),
            dynamic_map: dynamic_map.clone(),
            ..Default::default()
        },
        compiled_subroutines: AHashMap::new(),
        allocator: RegisterAllocator::new(),
    };
    compiler.compile_main(expr)?;
    Ok(compiler.program)
}

impl<'a> BytecodeCompiler<'a> {
    fn compile_main(&mut self, expr: &Expression) -> Result<(), BackendError> {
        self.allocator = RegisterAllocator::new();
        let mut main_bc = Vec::new();
        let final_reg = self.compile_recursive(expr, &mut main_bc, &HashSet::new())?;
        // The final result must be in R0 for the VM.
        if final_reg != 0 {
            main_bc.push(OpCode::Move(0, final_reg));
        }
        self.allocator.free(final_reg);
        main_bc.push(OpCode::Halt);
        self.program.main = main_bc;
        Ok(())
    }

    fn compile_subroutine(&mut self, id: u64) -> Result<(), BackendError> {
        if self.compiled_subroutines.contains_key(&id) {
            return Ok(());
        }
        let expr = self.definitions.get(&id).ok_or_else(|| {
            BackendError::InvalidLogic(format!("CSE Reference ID #{} not found", id))
        })?;
        self.compiled_subroutines.insert(id, ());
        let mut subroutine_bc = Vec::new();
        self.allocator = RegisterAllocator::new();
        let final_reg = self.compile_recursive(expr, &mut subroutine_bc, &HashSet::new())?;
        if final_reg != 0 {
            subroutine_bc.push(OpCode::Move(0, final_reg));
        }
        self.allocator.free(final_reg);
        subroutine_bc.push(OpCode::Return);
        self.program.subroutines.insert(id, subroutine_bc);
        Ok(())
    }

    fn compile_recursive(
        &mut self,
        expr: &Expression,
        bytecode: &mut Vec<OpCode>,
        live_after: &HashSet<Register>,
    ) -> Result<Register, BackendError> {
        match expr {
            Expression::Literal(val) => {
                let dest = self.allocator.alloc()?;
                bytecode.push(OpCode::LoadLiteral(dest, val.clone()));
                Ok(dest)
            }
            Expression::Input(source) => {
                let dest = self.allocator.alloc()?;
                let op = match source {
                    InputSource::Static { id } => OpCode::LoadStatic(dest, *id),
                    InputSource::Dynamic { id } => OpCode::LoadDynamic(dest, *id),
                    _ => {
                        return Err(BackendError::InvalidLogic(
                            "Encountered uninterned InputSource".to_string(),
                        ));
                    }
                };
                bytecode.push(op);
                Ok(dest)
            }
            Expression::Reference(id) => self.compile_call(id, bytecode, live_after),
            Expression::Not(v) => self.compile_unary(v, OpCode::Not, bytecode, live_after),
            Expression::Abs(v) => self.compile_unary(v, OpCode::Abs, bytecode, live_after),
            Expression::And(l, r) => self.compile_short_circuit(l, r, false, bytecode, live_after),
            Expression::Or(l, r) => self.compile_short_circuit(l, r, true, bytecode, live_after),
            _ => self.compile_binary_fallback(expr, bytecode, live_after),
        }
    }

    fn compile_unary<F>(
        &mut self,
        expr: &Expression,
        op_builder: F,
        bytecode: &mut Vec<OpCode>,
        live_after: &HashSet<Register>,
    ) -> Result<Register, BackendError>
    where
        F: Fn(Register, Register) -> OpCode,
    {
        let src = self.compile_recursive(expr, bytecode, live_after)?;
        // Optimization: if the source register is not live after this operation,
        // we can perform the operation in-place.
        let dest = if !live_after.contains(&src) {
            src
        } else {
            self.allocator.alloc()?
        };
        bytecode.push(op_builder(dest, src));
        Ok(dest)
    }

    fn compile_binary_fallback(
        &mut self,
        expr: &Expression,
        bytecode: &mut Vec<OpCode>,
        live_after: &HashSet<Register>,
    ) -> Result<Register, BackendError> {
        let (l, r, op_builder): (
            &Expression,
            &Expression,
            Box<dyn Fn(Register, Register, Register) -> OpCode>,
        ) = match expr {
            Expression::Sum(l, r) => (l, r, Box::new(OpCode::Add)),
            Expression::Subtract(l, r) => (l, r, Box::new(OpCode::Subtract)),
            Expression::Multiply(l, r) => (l, r, Box::new(OpCode::Multiply)),
            Expression::Divide(l, r) => (l, r, Box::new(OpCode::Divide)),
            Expression::Equal(l, r) => (l, r, Box::new(OpCode::Equal)),
            Expression::NotEqual(l, r) => (l, r, Box::new(OpCode::NotEqual)),
            Expression::GreaterThan(l, r) => (l, r, Box::new(OpCode::GreaterThan)),
            Expression::SmallerThan(l, r) => (l, r, Box::new(OpCode::LessThan)),
            Expression::GreaterThanOrEqual(l, r) => (l, r, Box::new(OpCode::GreaterThanOrEqual)),
            Expression::SmallerThanOrEqual(l, r) => (l, r, Box::new(OpCode::LessThanOrEqual)),
            Expression::Xor(l, r) => (l, r, Box::new(OpCode::Xor)),
            _ => {
                return Err(BackendError::UnsupportedAstNode(
                    "Unsupported binary expression".to_string(),
                ));
            }
        };

        let reg_l = self.compile_recursive(l, bytecode, live_after)?;
        let mut live_for_r = live_after.clone();
        live_for_r.insert(reg_l);
        let reg_r = self.compile_recursive(r, bytecode, &live_for_r)?;

        // Optimization: Try to use one of the source registers as the destination
        // to avoid allocating a new one.
        let dest = if !live_after.contains(&reg_l) {
            reg_l
        } else if !live_after.contains(&reg_r) {
            reg_r
        } else {
            self.allocator.alloc()?
        };

        bytecode.push(op_builder(dest, reg_l, reg_r));

        // Free the registers that are no longer live
        if dest != reg_l && !live_after.contains(&reg_l) {
            self.allocator.free(reg_l);
        }
        if dest != reg_r && !live_after.contains(&reg_r) {
            self.allocator.free(reg_r);
        }
        Ok(dest)
    }

    fn compile_call(
        &mut self,
        id: &u64,
        bytecode: &mut Vec<OpCode>,
        _live_after: &HashSet<Register>,
    ) -> Result<Register, BackendError> {
        self.compile_subroutine(*id)?;
        let dest = self.allocator.alloc()?;
        bytecode.push(OpCode::Call(*id));
        // The result of a subroutine is always placed in R0 by convention.
        bytecode.push(OpCode::Move(dest, 0));
        Ok(dest)
    }

    /// Compiles the short circuit. It compiles the left and right sides onlye once.
    fn compile_short_circuit(
        &mut self,
        l: &Expression,
        r: &Expression,
        is_or: bool,
        bytecode: &mut Vec<OpCode>,
        live_after: &HashSet<Register>,
    ) -> Result<Register, BackendError> {
        // 1. Compile the left-hand side. Its result register will hold the final value.
        let result_reg = self.compile_recursive(l, bytecode, live_after)?;

        // 2. Add a conditional jump.
        // For OR, we jump to the end if the result is true (short-circuit).
        // For AND, we jump to the end if the result is false (short-circuit).
        let jump_op = if is_or {
            OpCode::JumpIfTrue(result_reg, 0) // Placeholder address
        } else {
            OpCode::JumpIfFalse(result_reg, 0) // Placeholder address
        };
        bytecode.push(jump_op);
        let jump_idx = bytecode.len() - 1;

        // 3. Compile the right-hand side. This code is only executed if we don't short-circuit.
        // Note: `result_reg` must be kept alive for the evaluation of R.
        let mut live_for_r = live_after.clone();
        live_for_r.insert(result_reg);
        let reg_r = self.compile_recursive(r, bytecode, &live_for_r)?;

        // 4. Move the result of the RHS into our final result register.
        bytecode.push(OpCode::Move(result_reg, reg_r));
        if !live_after.contains(&reg_r) {
            self.allocator.free(reg_r);
        }

        // 5. The short-circuit jump from step 2 should land here, at the end of the expression.
        let target_addr = bytecode.len() as Address;

        // 6. Patch the jump instruction with the correct target address.
        match &mut bytecode[jump_idx] {
            OpCode::JumpIfTrue(_, addr) | OpCode::JumpIfFalse(_, addr) => *addr = target_addr,
            _ => unreachable!(),
        };

        // The final result is in `result_reg`.
        Ok(result_reg)
    }
}
