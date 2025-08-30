use crate::ast::Value;
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

/// A stack-based Virtual Machine for executing Hantei bytecode.
pub struct Vm<'a> {
    bytecode: &'a [OpCode],
    ip: usize,
    stack: Vec<Value>,
    static_data: &'a AHashMap<String, f64>,
    dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
}

impl<'a> Vm<'a> {
    pub fn new(
        bytecode: &'a [OpCode],
        static_data: &'a AHashMap<String, f64>,
        dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Self {
        Self {
            bytecode,
            ip: 0,
            stack: Vec::with_capacity(16),
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
                OpCode::Push(val) => self.push(val.clone()),
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Return => return self.pop(),

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
