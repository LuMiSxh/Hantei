use crate::ast::Value;
use crate::bytecode::compiler::BytecodeProgram;
use crate::bytecode::opcode::{OpCode, Register};
use crate::error::VmError;
use ahash::AHashMap;

const NUM_REGISTERS: usize = 256;

macro_rules! binary_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = $self.get_reg($src1)?;
        let v2 = $self.get_reg($src2)?;
        match (v1, v2) {
            (Value::Number(l), Value::Number(r)) => {
                $self.set_reg($dest, Value::Number(*l $op *r));
                Ok(())
            }
            (l, _r) => Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l.clone() }),
        }
    }};
}

macro_rules! comparison_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = $self.get_reg($src1)?;
        let v2 = $self.get_reg($src2)?;
        match (v1, v2) {
            (Value::Number(l), Value::Number(r)) => {
                $self.set_reg($dest, Value::Bool(*l $op *r));
                Ok(())
            }
            (l, _r) => Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l.clone() }),
        }
    }};
}

macro_rules! logical_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = $self.get_reg($src1)?;
        let v2 = $self.get_reg($src2)?;
        match (v1, v2) {
            (Value::Bool(l), Value::Bool(r)) => {
                $self.set_reg($dest, Value::Bool(*l $op *r));
                Ok(())
            }
            (l, _) => Err(VmError::TypeMismatch { expected: "Bool".to_string(), found: l.clone() }),
        }
    }};
}

pub struct Vm<'a> {
    program: &'a BytecodeProgram,
    ip: usize,
    bytecode: &'a [OpCode],
    registers: [Value; NUM_REGISTERS],
    call_stack: Vec<(usize, &'a [OpCode])>,
    static_data: &'a AHashMap<String, f64>,
    dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
}

impl<'a> Vm<'a> {
    pub fn new(
        program: &'a BytecodeProgram,
        static_data: &'a AHashMap<String, f64>,
        dynamic_context: &'a AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Self {
        Self {
            program,
            ip: 0,
            bytecode: &program.main,
            registers: std::array::from_fn(|_| Value::Null),
            call_stack: Vec::with_capacity(8),
            static_data,
            dynamic_context,
        }
    }

    #[inline(always)]
    fn get_reg(&self, reg: Register) -> Result<&Value, VmError> {
        self.registers
            .get(reg as usize)
            .ok_or(VmError::InvalidRegister(reg))
    }

    #[inline(always)]
    fn set_reg(&mut self, reg: Register, val: Value) {
        self.registers[reg as usize] = val;
    }

    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            let instruction = self
                .bytecode
                .get(self.ip)
                .ok_or(VmError::InvalidIp(self.ip))?;
            self.ip += 1;

            match *instruction {
                OpCode::Halt => return Ok(self.get_reg(0)?.clone()),
                OpCode::LoadLiteral(dest, ref val) => self.set_reg(dest, val.clone()),
                OpCode::LoadStatic(dest, ref name) => {
                    let val = self
                        .static_data
                        .get(name)
                        .map(|v| Value::Number(*v))
                        .ok_or_else(|| VmError::InputNotFound(name.clone()))?;
                    self.set_reg(dest, val);
                }
                OpCode::LoadDynamic(dest, ref event, ref field) => {
                    let val = self
                        .dynamic_context
                        .get(event)
                        .and_then(|data| data.get(field))
                        .map(|v| Value::Number(*v))
                        .unwrap_or(Value::Null);
                    self.set_reg(dest, val);
                }
                OpCode::Move(dest, src) => self.set_reg(dest, self.get_reg(src)?.clone()),
                OpCode::Add(dest, src1, src2) => binary_op!(self, dest, src1, src2, +)?,
                OpCode::Subtract(dest, src1, src2) => binary_op!(self, dest, src1, src2, -)?,
                OpCode::Multiply(dest, src1, src2) => binary_op!(self, dest, src1, src2, *)?,
                OpCode::Divide(dest, src1, src2) => binary_op!(self, dest, src1, src2, /)?,
                OpCode::Xor(dest, src1, src2) => logical_op!(self, dest, src1, src2, ^)?,
                OpCode::Abs(dest, src) => {
                    if let Value::Number(n) = self.get_reg(src)? {
                        self.set_reg(dest, Value::Number(n.abs()));
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "Number".to_string(),
                            found: self.get_reg(src)?.clone(),
                        });
                    }
                }
                OpCode::Not(dest, src) => {
                    if let Value::Bool(b) = self.get_reg(src)? {
                        self.set_reg(dest, Value::Bool(!b));
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "Bool".to_string(),
                            found: self.get_reg(src)?.clone(),
                        });
                    }
                }
                OpCode::Equal(dest, src1, src2) => self.set_reg(
                    dest,
                    Value::Bool(self.get_reg(src1)? == self.get_reg(src2)?),
                ),
                OpCode::NotEqual(dest, src1, src2) => self.set_reg(
                    dest,
                    Value::Bool(self.get_reg(src1)? != self.get_reg(src2)?),
                ),
                OpCode::GreaterThan(dest, src1, src2) => comparison_op!(self, dest, src1, src2, >)?,
                OpCode::LessThan(dest, src1, src2) => comparison_op!(self, dest, src1, src2, <)?,
                OpCode::GreaterThanOrEqual(dest, src1, src2) => {
                    comparison_op!(self, dest, src1, src2, >=)?
                }
                OpCode::LessThanOrEqual(dest, src1, src2) => {
                    comparison_op!(self, dest, src1, src2, <=)?
                }
                OpCode::Jump(addr) => self.ip = addr as usize,
                OpCode::JumpIfFalse(reg, addr) => {
                    if let Value::Bool(false) = self.get_reg(reg)? {
                        self.ip = addr as usize;
                    }
                }
                OpCode::JumpIfTrue(reg, addr) => {
                    if let Value::Bool(true) = self.get_reg(reg)? {
                        self.ip = addr as usize;
                    }
                }
                OpCode::Call(id) => {
                    self.call_stack.push((self.ip, self.bytecode));
                    self.bytecode = self
                        .program
                        .subroutines
                        .get(&id)
                        .ok_or_else(|| VmError::UnknownSubroutine(id))?;
                    self.ip = 0;
                }
                OpCode::Return => {
                    let (ret_ip, prev_bytecode) =
                        self.call_stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.ip = ret_ip;
                    self.bytecode = prev_bytecode;
                }
            }
        }
    }
}
