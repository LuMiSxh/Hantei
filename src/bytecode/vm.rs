use crate::ast::Value;
use crate::bytecode::compiler::BytecodeProgram;
use crate::bytecode::opcode::OpCode;
use crate::error::VmError;
use ahash::AHashMap;

macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        {
            let right = $self.pop()?;
            let left = $self.pop()?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => $self.push(Value::Number(l $op r)),
                (l, _) => return Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l })
            }
        }
    };
}

macro_rules! comparison_op {
    ($self:ident, $op:tt) => {
        {
            let right = $self.pop()?;
            let left = $self.pop()?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => $self.push(Value::Bool(l $op r)),
                (l, _) => return Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l })
            }
        }
    };
}

/// A stack-based Virtual Machine with support for subroutines.
pub(super) struct Vm<'a> {
    program: &'a BytecodeProgram,
    ip: usize,
    // The currently executing chunk of bytecode (either main or a subroutine).
    bytecode: &'a [OpCode],
    stack: Vec<Value>,
    // A stack of instruction pointers to return to after a Call.
    call_stack: Vec<usize>,
    static_data: &'a AHashMap<String, f64>,
    dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
}

impl<'a> Vm<'a> {
    pub(super) fn new(
        program: &'a BytecodeProgram,
        static_data: &'a AHashMap<String, f64>,
        dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Self {
        Self {
            program,
            ip: 0,
            bytecode: &program.main, // Start execution in the main program
            stack: Vec::with_capacity(16),
            call_stack: Vec::with_capacity(8),
            static_data,
            dynamic_context,
        }
    }

    /// Runs the bytecode until a `Return` instruction is encountered.
    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            let instruction = self
                .bytecode
                .get(self.ip)
                .ok_or(VmError::InvalidIp(self.ip))?;
            self.ip += 1;

            match instruction {
                OpCode::Halt => return self.pop(),

                // --- Subroutine Instructions ---
                OpCode::Call(id) => {
                    self.call_stack.push(self.ip); // Store return address
                    self.bytecode = self
                        .program
                        .subroutines
                        .get(id)
                        .ok_or_else(|| VmError::UnknownSubroutine(*id))?;
                    self.ip = 0; // Jump to start of subroutine
                }
                OpCode::Return => {
                    self.ip = self.call_stack.pop().ok_or(VmError::StackUnderflow)?; // Get return address
                    self.bytecode = &self.program.main; // Assume calls are only from main for now
                }

                // --- Stack Operations ---
                OpCode::Push(val) => self.push(val.clone()),
                OpCode::Pop => {
                    self.pop()?;
                }

                // --- Data Loading ---
                OpCode::LoadStatic(name) => {
                    let val = self
                        .static_data
                        .get(name)
                        .map(|v| Value::Number(*v))
                        .ok_or_else(|| VmError::InputNotFound(name.clone()))?;
                    self.push(val);
                }
                OpCode::LoadDynamic(event, field) => {
                    let val = self
                        .dynamic_context
                        .get(event)
                        .and_then(|data| data.get(field))
                        .map(|v| Value::Number(*v))
                        .ok_or_else(|| VmError::InputNotFound(format!("{}.{}", event, field)))?;
                    self.push(val);
                }

                // --- Operators ---
                OpCode::Add => binary_op!(self, +),
                OpCode::Subtract => binary_op!(self, -),
                OpCode::Multiply => binary_op!(self, *),
                OpCode::Divide => binary_op!(self, /),
                OpCode::GreaterThan => comparison_op!(self, >),
                OpCode::LessThan => comparison_op!(self, <),
                OpCode::GreaterThanOrEqual => comparison_op!(self, >=),
                OpCode::LessThanOrEqual => comparison_op!(self, <=),
                OpCode::Equal => {
                    let r = self.pop()?;
                    let l = self.pop()?;
                    self.push(Value::Bool(l == r));
                }
                OpCode::NotEqual => {
                    let r = self.pop()?;
                    let l = self.pop()?;
                    self.push(Value::Bool(l != r));
                }
                OpCode::Not => {
                    let val = self.pop()?;
                    if let Value::Bool(b) = val {
                        self.push(Value::Bool(!b));
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "Bool".to_string(),
                            found: val,
                        });
                    }
                }

                // --- Control Flow ---
                OpCode::Jump(addr) => self.ip = *addr,
                OpCode::JumpIfFalse(addr) => {
                    let val = self.stack.last().ok_or(VmError::StackUnderflow)?;
                    if let Value::Bool(false) = val {
                        self.ip = *addr;
                    }
                }
                OpCode::JumpIfTrue(addr) => {
                    let val = self.stack.last().ok_or(VmError::StackUnderflow)?;
                    if let Value::Bool(true) = val {
                        self.ip = *addr;
                    }
                }
                _ => return Err(VmError::UnhandledOpCode(instruction.to_owned())),
            }
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }
}
