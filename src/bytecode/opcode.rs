use crate::ast::Value;

/// An instruction for the stack-based virtual machine.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // Stack Operations
    Push(Value),
    Pop,

    // Data Loading
    LoadStatic(String),
    LoadDynamic(String, String),

    // Arithmetic & Unary Operators
    Add,
    Subtract,
    Multiply,
    Divide,
    Abs,
    Not,

    // Comparison & Equality Operators
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,

    // Logical Operators
    And,
    Or,
    Xor,

    // Control Flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),

    /// Call a subroutine located at a specific ID.
    Call(u64),
    /// Return from the current subroutine to the last call site.
    Return,
    /// Stop execution of the VM completely.
    Halt,
}
