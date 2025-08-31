pub use crate::ast::InputId;
use crate::ast::Value;
use serde::{Deserialize, Serialize};

pub type Register = u8;
pub type Address = u16; // Up to 65536 instructions per chunk
pub type SubroutineId = u64;

/// An instruction for the register-based virtual machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // Data Loading (0-3)
    // src, dest
    LoadLiteral(Register, Value),
    LoadStatic(Register, InputId),
    LoadDynamic(Register, InputId),
    Move(Register, Register),

    // Arithmetic (4-8)
    // dest, src1, src2
    Add(Register, Register, Register),
    Subtract(Register, Register, Register),
    Multiply(Register, Register, Register),
    Divide(Register, Register, Register),
    Xor(Register, Register, Register),

    // Unary (9-10)
    // dest, src
    Abs(Register, Register),
    Not(Register, Register),

    // Comparison & Equality (11-16)
    // dest, src1, src2
    Equal(Register, Register, Register),
    NotEqual(Register, Register, Register),
    GreaterThan(Register, Register, Register),
    LessThan(Register, Register, Register),
    GreaterThanOrEqual(Register, Register, Register),
    LessThanOrEqual(Register, Register, Register),

    // Fusion of Comparison & Control Flow (17-22)
    // src1, src2, address
    JumpIfEq(Register, Register, Address),
    JumpIfNeq(Register, Register, Address),
    JumpIfGt(Register, Register, Address),
    JumpIfGte(Register, Register, Address),
    JumpIfLt(Register, Register, Address),
    JumpIfLte(Register, Register, Address),

    // Control Flow (23-25)
    // address
    Jump(Address),
    // src, address
    JumpIfFalse(Register, Address),
    JumpIfTrue(Register, Address),

    // Subroutines (26-27)
    Call(SubroutineId),
    Return,

    // VM Control (28)
    Halt,
}
