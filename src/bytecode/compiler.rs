use crate::ast::{Expression, InputSource};
use crate::bytecode::opcode::{Address, InputId, OpCode, Register};
use crate::error::BackendError;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

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
    next_register: Register,
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
        next_register: 0,
    };
    compiler.compile_main(expr)?;
    Ok(compiler.program)
}

impl<'a> BytecodeCompiler<'a> {
    fn reset_allocator(&mut self) {
        self.next_register = 0;
    }

    fn alloc_reg(&mut self) -> Result<Register, BackendError> {
        let reg = self.next_register;
        self.next_register = self.next_register.checked_add(1).ok_or_else(|| {
            BackendError::ResourceLimitExceeded("Register limit reached".to_string())
        })?;
        Ok(reg)
    }

    fn compile_main(&mut self, expr: &Expression) -> Result<(), BackendError> {
        self.reset_allocator();
        let mut main_bc = Vec::new();
        let final_reg = self.compile_recursive(expr, &mut main_bc)?;
        if final_reg != 0 {
            main_bc.push(OpCode::Move(0, final_reg));
        }
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
        self.reset_allocator();
        let final_reg = self.compile_recursive(expr, &mut subroutine_bc)?;
        if final_reg != 0 {
            subroutine_bc.push(OpCode::Move(0, final_reg));
        }
        subroutine_bc.push(OpCode::Return);
        self.program.subroutines.insert(id, subroutine_bc);
        Ok(())
    }

    fn compile_recursive(
        &mut self,
        expr: &Expression,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<Register, BackendError> {
        match expr {
            Expression::Literal(val) => {
                let dest = self.alloc_reg()?;
                bytecode.push(OpCode::LoadLiteral(dest, val.clone()));
                Ok(dest)
            }
            Expression::Input(source) => {
                let dest = self.alloc_reg()?;
                let op = match source {
                    // Runtime variants - these should be the only ones present after interning
                    InputSource::Static { id } => OpCode::LoadStatic(dest, *id),
                    InputSource::Dynamic { id } => OpCode::LoadDynamic(dest, *id),
                    // Compilation-time variants - these should not reach bytecode compilation
                    InputSource::StaticName { name } => {
                        return Err(BackendError::InvalidLogic(format!(
                            "Encountered uninterned static input '{}' during bytecode compilation",
                            name
                        )));
                    }
                    InputSource::DynamicName { event, field } => {
                        return Err(BackendError::InvalidLogic(format!(
                            "Encountered uninterned dynamic input '{}.{}' during bytecode compilation",
                            event, field
                        )));
                    }
                };
                bytecode.push(op);
                Ok(dest)
            }
            Expression::Reference(id) => {
                self.compile_subroutine(*id)?;
                bytecode.push(OpCode::Call(*id));
                let dest = self.alloc_reg()?;
                bytecode.push(OpCode::Move(dest, 0));
                Ok(dest)
            }
            Expression::Abs(val) => {
                let src = self.compile_recursive(val, bytecode)?;
                let dest = self.alloc_reg()?;
                bytecode.push(OpCode::Abs(dest, src));
                Ok(dest)
            }
            Expression::Not(val) => {
                let src = self.compile_recursive(val, bytecode)?;
                let dest = self.alloc_reg()?;
                bytecode.push(OpCode::Not(dest, src));
                Ok(dest)
            }
            Expression::Sum(l, r) => self.compile_binary(l, r, OpCode::Add, bytecode),
            Expression::Subtract(l, r) => self.compile_binary(l, r, OpCode::Subtract, bytecode),
            Expression::Multiply(l, r) => self.compile_binary(l, r, OpCode::Multiply, bytecode),
            Expression::Divide(l, r) => self.compile_binary(l, r, OpCode::Divide, bytecode),
            Expression::Equal(l, r) => self.compile_binary(l, r, OpCode::Equal, bytecode),
            Expression::NotEqual(l, r) => self.compile_binary(l, r, OpCode::NotEqual, bytecode),
            Expression::GreaterThan(l, r) => {
                self.compile_binary(l, r, OpCode::GreaterThan, bytecode)
            }
            Expression::SmallerThan(l, r) => self.compile_binary(l, r, OpCode::LessThan, bytecode),
            Expression::GreaterThanOrEqual(l, r) => {
                self.compile_binary(l, r, OpCode::GreaterThanOrEqual, bytecode)
            }
            Expression::SmallerThanOrEqual(l, r) => {
                self.compile_binary(l, r, OpCode::LessThanOrEqual, bytecode)
            }
            Expression::Xor(l, r) => self.compile_binary(l, r, OpCode::Xor, bytecode),
            Expression::And(l, r) => self.compile_short_circuit(l, r, false, bytecode),
            Expression::Or(l, r) => self.compile_short_circuit(l, r, true, bytecode),
        }
    }

    fn compile_binary<F>(
        &mut self,
        l: &Expression,
        r: &Expression,
        op_builder: F,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<Register, BackendError>
    where
        F: Fn(Register, Register, Register) -> OpCode,
    {
        let reg_l = self.compile_recursive(l, bytecode)?;
        let reg_r = self.compile_recursive(r, bytecode)?;
        let dest = self.alloc_reg()?;
        bytecode.push(op_builder(dest, reg_l, reg_r));
        Ok(dest)
    }

    fn compile_short_circuit(
        &mut self,
        l: &Expression,
        r: &Expression,
        is_or: bool,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<Register, BackendError> {
        let result_reg = self.alloc_reg()?;
        let reg_l = self.compile_recursive(l, bytecode)?;

        let jump_idx = bytecode.len();
        bytecode.push(OpCode::Jump(0)); // Placeholder

        let reg_r = self.compile_recursive(r, bytecode)?;
        bytecode.push(OpCode::Move(result_reg, reg_r));
        let jump_to_end_idx = bytecode.len();
        bytecode.push(OpCode::Jump(0));

        let short_circuit_addr = bytecode.len() as Address;
        bytecode.push(OpCode::Move(result_reg, reg_l));

        let end_addr = bytecode.len() as Address;

        bytecode[jump_to_end_idx] = OpCode::Jump(end_addr);
        if is_or {
            bytecode[jump_idx] = OpCode::JumpIfTrue(reg_l, short_circuit_addr);
        } else {
            bytecode[jump_idx] = OpCode::JumpIfFalse(reg_l, short_circuit_addr);
        }

        Ok(result_reg)
    }
}
