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

    // End of execution
    Return,
}
