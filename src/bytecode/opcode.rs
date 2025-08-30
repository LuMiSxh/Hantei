use crate::ast::Value;

pub type Register = u8;
pub type Address = u16; // Up to 65536 instructions per chunk
pub type SubroutineId = u64;

/// An instruction for the register-based virtual machine.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // Data Loading
    LoadLiteral(Register, Value),
    LoadStatic(Register, String),
    LoadDynamic(Register, String, String),
    Move(Register, Register), // Move value from one register to another

    // Arithmetic
    Add(Register, Register, Register),      // dest, src1, src2
    Subtract(Register, Register, Register), // dest, src1, src2
    Multiply(Register, Register, Register), // dest, src1, src2
    Divide(Register, Register, Register),   // dest, src1, src2
    Xor(Register, Register, Register),      // dest, src1, src2 - Logical XOR for booleans

    // Unary
    Abs(Register, Register), // dest, src
    Not(Register, Register), // dest, src

    // Comparison & Equality (result is always a Bool in dest)
    Equal(Register, Register, Register),
    NotEqual(Register, Register, Register),
    GreaterThan(Register, Register, Register),
    LessThan(Register, Register, Register),
    GreaterThanOrEqual(Register, Register, Register),
    LessThanOrEqual(Register, Register, Register),

    // Control Flow
    Jump(Address),
    JumpIfFalse(Register, Address), // Jumps if the value in the register is false
    JumpIfTrue(Register, Address),  // Jumps if the value in the register is true

    // Subroutines
    Call(SubroutineId),
    Return,

    // VM Control
    Halt,
}
