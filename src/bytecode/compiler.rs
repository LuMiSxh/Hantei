use crate::ast::{Expression, InputSource};
use crate::bytecode::opcode::OpCode;
use crate::error::BackendError;
use ahash::AHashMap;

/// Represents a fully compiled bytecode program, including subroutines.
#[derive(Debug, Default)]
pub struct BytecodeProgram {
    pub main: Vec<OpCode>,
    pub subroutines: AHashMap<u64, Vec<OpCode>>,
}

/// A stateful compiler that translates an AST and its definitions into a `BytecodeProgram`.
pub struct BytecodeCompiler<'a> {
    definitions: &'a AHashMap<u64, Expression>,
    program: BytecodeProgram,
    // Tracks which subroutine IDs have already been compiled to prevent redundant work.
    compiled_subroutines: AHashMap<u64, ()>,
}

/// Main entry point for compiling an AST.
pub fn compile_to_program(
    expr: &Expression,
    definitions: &AHashMap<u64, Expression>,
) -> Result<BytecodeProgram, BackendError> {
    let mut compiler = BytecodeCompiler {
        definitions,
        program: BytecodeProgram::default(),
        compiled_subroutines: AHashMap::new(),
    };
    compiler.compile_main(expr)?;
    Ok(compiler.program)
}

impl<'a> BytecodeCompiler<'a> {
    /// Compiles the main entry point of the program.
    fn compile_main(&mut self, expr: &Expression) -> Result<(), BackendError> {
        let mut main_bc = Vec::new();
        self.compile_recursive(expr, &mut main_bc)?;
        main_bc.push(OpCode::Halt);
        self.program.main = main_bc;
        Ok(())
    }

    /// Compiles a subroutine for a given ID.
    fn compile_subroutine(&mut self, id: u64) -> Result<(), BackendError> {
        // If we have already compiled this subroutine, do nothing.
        if self.compiled_subroutines.contains_key(&id) {
            return Ok(());
        }

        let expr = self.definitions.get(&id).ok_or_else(|| {
            BackendError::InvalidLogic(format!("CSE Reference ID #{} not found", id))
        })?;

        // Temporarily insert a marker to handle recursive subroutines if they ever occur.
        self.compiled_subroutines.insert(id, ());

        let mut subroutine_bc = Vec::new();
        self.compile_recursive(expr, &mut subroutine_bc)?;
        subroutine_bc.push(OpCode::Return);

        // Store the final compiled subroutine.
        self.program.subroutines.insert(id, subroutine_bc);
        Ok(())
    }

    /// The core recursive compilation logic.
    fn compile_recursive(
        &mut self,
        expr: &Expression,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BackendError> {
        match expr {
            // --- Leaf Nodes & Subroutine Calls ---
            Expression::Literal(val) => bytecode.push(OpCode::Push(val.clone())),

            Expression::Input(source) => match source {
                InputSource::Static { name } => bytecode.push(OpCode::LoadStatic(name.clone())),
                InputSource::Dynamic { event, field } => {
                    bytecode.push(OpCode::LoadDynamic(event.clone(), field.clone()))
                }
            },

            Expression::Reference(id) => {
                // Ensure the subroutine for this ID is compiled.
                self.compile_subroutine(*id)?;
                // Emit a single instruction to call it.
                bytecode.push(OpCode::Call(*id));
            }

            // --- Unary Operators ---
            Expression::Abs(val) => {
                self.compile_recursive(val, bytecode)?;
                bytecode.push(OpCode::Abs);
            }
            Expression::Not(val) => {
                self.compile_recursive(val, bytecode)?;
                bytecode.push(OpCode::Not);
            }

            // --- Binary Operators (handled by helper) ---
            Expression::Sum(l, r) => self.compile_binary(l, r, OpCode::Add, bytecode)?,
            Expression::Subtract(l, r) => self.compile_binary(l, r, OpCode::Subtract, bytecode)?,
            Expression::Multiply(l, r) => self.compile_binary(l, r, OpCode::Multiply, bytecode)?,
            Expression::Divide(l, r) => self.compile_binary(l, r, OpCode::Divide, bytecode)?,
            Expression::Equal(l, r) => self.compile_binary(l, r, OpCode::Equal, bytecode)?,
            Expression::NotEqual(l, r) => self.compile_binary(l, r, OpCode::NotEqual, bytecode)?,
            Expression::GreaterThan(l, r) => {
                self.compile_binary(l, r, OpCode::GreaterThan, bytecode)?
            }
            Expression::SmallerThan(l, r) => {
                self.compile_binary(l, r, OpCode::LessThan, bytecode)?
            }
            Expression::GreaterThanOrEqual(l, r) => {
                self.compile_binary(l, r, OpCode::GreaterThanOrEqual, bytecode)?
            }
            Expression::SmallerThanOrEqual(l, r) => {
                self.compile_binary(l, r, OpCode::LessThanOrEqual, bytecode)?
            }
            Expression::Xor(l, r) => self.compile_binary(l, r, OpCode::Xor, bytecode)?,

            // --- Logical Operators with Short-Circuiting ---
            Expression::And(l, r) => {
                self.compile_recursive(l, bytecode)?;
                let jump_idx = bytecode.len();
                bytecode.push(OpCode::JumpIfFalse(0)); // Placeholder
                bytecode.push(OpCode::Pop); // Pop the `true` from the left side
                self.compile_recursive(r, bytecode)?;
                let jump_target = bytecode.len();
                bytecode[jump_idx] = OpCode::JumpIfFalse(jump_target);
            }
            Expression::Or(l, r) => {
                self.compile_recursive(l, bytecode)?;
                let jump_idx = bytecode.len();
                bytecode.push(OpCode::JumpIfTrue(0)); // Placeholder
                bytecode.push(OpCode::Pop); // Pop the `false` from the left side
                self.compile_recursive(r, bytecode)?;
                let jump_target = bytecode.len();
                bytecode[jump_idx] = OpCode::JumpIfTrue(jump_target);
            }
        }
        Ok(())
    }

    /// Helper function to compile standard binary expressions.
    fn compile_binary(
        &mut self,
        l: &Expression,
        r: &Expression,
        op: OpCode,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BackendError> {
        self.compile_recursive(l, bytecode)?;
        self.compile_recursive(r, bytecode)?;
        bytecode.push(op);
        Ok(())
    }
}
