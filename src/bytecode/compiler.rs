use crate::ast::{Expression, InputSource};
use crate::bytecode::opcode::OpCode;
use crate::error::BackendError;

/// Compiles a single expression AST into a vector of bytecode instructions.
pub fn compile_ast(expr: &Expression) -> Result<Vec<OpCode>, BackendError> {
    let mut bytecode = Vec::new();
    compile_recursive(expr, &mut bytecode)?;
    bytecode.push(OpCode::Return);
    Ok(bytecode)
}

/// Helper to recursively compile expressions.
fn compile_recursive(expr: &Expression, bytecode: &mut Vec<OpCode>) -> Result<(), BackendError> {
    match expr {
        Expression::Literal(val) => bytecode.push(OpCode::Push(val.clone())),
        Expression::Input(source) => match source {
            InputSource::Static { name } => bytecode.push(OpCode::LoadStatic(name.clone())),
            InputSource::Dynamic { event, field } => {
                bytecode.push(OpCode::LoadDynamic(event.clone(), field.clone()))
            }
        },

        // --- Binary Operators ---
        Expression::Sum(l, r) => compile_binary(l, r, OpCode::Add, bytecode)?,
        Expression::Subtract(l, r) => compile_binary(l, r, OpCode::Subtract, bytecode)?,
        Expression::Multiply(l, r) => compile_binary(l, r, OpCode::Multiply, bytecode)?,
        Expression::Divide(l, r) => compile_binary(l, r, OpCode::Divide, bytecode)?,
        Expression::Equal(l, r) => compile_binary(l, r, OpCode::Equal, bytecode)?,
        Expression::NotEqual(l, r) => compile_binary(l, r, OpCode::NotEqual, bytecode)?,
        Expression::GreaterThan(l, r) => compile_binary(l, r, OpCode::GreaterThan, bytecode)?,
        Expression::SmallerThan(l, r) => compile_binary(l, r, OpCode::LessThan, bytecode)?,
        Expression::GreaterThanOrEqual(l, r) => {
            compile_binary(l, r, OpCode::GreaterThanOrEqual, bytecode)?
        }
        Expression::SmallerThanOrEqual(l, r) => {
            compile_binary(l, r, OpCode::LessThanOrEqual, bytecode)?
        }
        Expression::Xor(l, r) => compile_binary(l, r, OpCode::Xor, bytecode)?,

        // --- Logical Operators with Short-Circuiting ---
        Expression::And(l, r) => {
            compile_recursive(l, bytecode)?;
            // If left is false, jump past the right side's code.
            let jump_idx = bytecode.len();
            bytecode.push(OpCode::JumpIfFalse(0)); // Placeholder address
            bytecode.push(OpCode::Pop); // Pop the `true` from the left side
            compile_recursive(r, bytecode)?;
            // Patch the jump address
            bytecode[jump_idx] = OpCode::JumpIfFalse(bytecode.len());
        }
        Expression::Or(l, r) => {
            compile_recursive(l, bytecode)?;
            // If left is true, jump past the right side's code.
            let jump_idx = bytecode.len();
            // We need a JumpIfTrue. Let's simulate it: NOT -> JumpIfFalse
            bytecode.push(OpCode::Not);
            bytecode.push(OpCode::JumpIfFalse(0)); // Placeholder for jump if original was true
            bytecode.push(OpCode::Pop); // Pop the `false` from the left side
            compile_recursive(r, bytecode)?;
            // Patch the jump address
            let jump_target = bytecode.len();
            let original_op = &bytecode[jump_idx + 1];
            if let OpCode::JumpIfFalse(_) = original_op {
                bytecode[jump_idx + 1] = OpCode::JumpIfFalse(jump_target);
            }
        }

        // --- Unary Operators ---
        Expression::Abs(val) => {
            compile_recursive(val, bytecode)?;
            bytecode.push(OpCode::Abs);
        }
        Expression::Not(val) => {
            compile_recursive(val, bytecode)?;
            bytecode.push(OpCode::Not);
        }
    }
    Ok(())
}

fn compile_binary(
    l: &Expression,
    r: &Expression,
    op: OpCode,
    bytecode: &mut Vec<OpCode>,
) -> Result<(), BackendError> {
    compile_recursive(l, bytecode)?;
    compile_recursive(r, bytecode)?;
    bytecode.push(op);
    Ok(())
}
