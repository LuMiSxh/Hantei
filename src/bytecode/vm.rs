use crate::ast::Value;
use crate::bytecode::compiler::BytecodeProgram;
use crate::bytecode::opcode::{OpCode, Register};
use crate::error::VmError;

/// Number of registers in the VM.
/// This is a fixed size for simplicity, but could be made dynamic if needed.
/// Must be <= 256 to fit in a single byte for register encoding.
const NUM_REGISTERS: usize = 64;

macro_rules! binary_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = unsafe { $self.get_reg_unchecked($src1) };
        let v2 = unsafe { $self.get_reg_unchecked($src2) };
        match (v1, v2) {
            (Value::Number(l), Value::Number(r)) => {
                unsafe { $self.set_reg_unchecked($dest, Value::Number(*l $op *r)) };
                Ok(())
            }
            (l, _r) => Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l.clone() }),
        }
    }};
}

macro_rules! comparison_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = unsafe { $self.get_reg_unchecked($src1) };
        let v2 = unsafe { $self.get_reg_unchecked($src2) };
        match (v1, v2) {
            (Value::Number(l), Value::Number(r)) => {
                unsafe { $self.set_reg_unchecked($dest, Value::Bool(*l $op *r)) };
                Ok(())
            }
            (l, _r) => Err(VmError::TypeMismatch { expected: "Number".to_string(), found: l.clone() }),
        }
    }};
}

macro_rules! logical_op {
    ($self:ident, $dest:ident, $src1:ident, $src2:ident, $op:tt) => {{
        let v1 = unsafe { $self.get_reg_unchecked($src1) };
        let v2 = unsafe { $self.get_reg_unchecked($src2) };
        match (v1, v2) {
            (Value::Bool(l), Value::Bool(r)) => {
                unsafe { $self.set_reg_unchecked($dest, Value::Bool(*l $op *r)) };
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
    static_data: &'a [Value],
    dynamic_context: &'a [Value],
}

impl<'a> Vm<'a> {
    pub fn new(
        program: &'a BytecodeProgram,
        static_data: &'a [Value],
        dynamic_context: &'a [Value],
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

    /// Unsafe, unchecked, and always-inlined register access.
    #[inline(always)]
    unsafe fn get_reg_unchecked(&self, reg: Register) -> &Value {
        unsafe { self.registers.get_unchecked(reg as usize) }
    }

    /// Unsafe, unchecked, and always-inlined register setting.
    #[inline(always)]
    unsafe fn set_reg_unchecked(&mut self, reg: Register, val: Value) {
        unsafe {
            *self.registers.get_unchecked_mut(reg as usize) = val;
        }
    }

    #[inline(always)]
    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            let instruction = unsafe { self.bytecode.get_unchecked(self.ip) };
            self.ip += 1;

            match *instruction {
                OpCode::Halt => return Ok(unsafe { self.get_reg_unchecked(0) }.clone()),
                OpCode::LoadLiteral(dest, ref val) => unsafe {
                    self.set_reg_unchecked(dest, val.clone())
                },
                OpCode::LoadStatic(dest, id) => {
                    let val = self
                        .static_data
                        .get(id as usize)
                        .ok_or(VmError::InputIdOutOfBounds(id))?;
                    unsafe { self.set_reg_unchecked(dest, val.clone()) };
                }
                OpCode::LoadDynamic(dest, id) => {
                    let val = self
                        .dynamic_context
                        .get(id as usize)
                        .ok_or(VmError::InputIdOutOfBounds(id))?;
                    unsafe { self.set_reg_unchecked(dest, val.clone()) };
                }
                OpCode::Move(dest, src) => {
                    let val = unsafe { self.get_reg_unchecked(src) }.clone();
                    unsafe { self.set_reg_unchecked(dest, val) };
                }
                OpCode::Add(dest, src1, src2) => binary_op!(self, dest, src1, src2, +)?,
                OpCode::Subtract(dest, src1, src2) => binary_op!(self, dest, src1, src2, -)?,
                OpCode::Multiply(dest, src1, src2) => binary_op!(self, dest, src1, src2, *)?,
                OpCode::Divide(dest, src1, src2) => binary_op!(self, dest, src1, src2, /)?,
                OpCode::Xor(dest, src1, src2) => logical_op!(self, dest, src1, src2, ^)?,
                OpCode::Abs(dest, src) => {
                    if let Value::Number(n) = unsafe { self.get_reg_unchecked(src) } {
                        unsafe { self.set_reg_unchecked(dest, Value::Number(n.abs())) };
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "Number".to_string(),
                            found: unsafe { self.get_reg_unchecked(src) }.clone(),
                        });
                    }
                }
                OpCode::Not(dest, src) => {
                    if let Value::Bool(b) = unsafe { self.get_reg_unchecked(src) } {
                        unsafe { self.set_reg_unchecked(dest, Value::Bool(!*b)) };
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "Bool".to_string(),
                            found: unsafe { self.get_reg_unchecked(src) }.clone(),
                        });
                    }
                }
                OpCode::Equal(dest, src1, src2) => {
                    let are_equal =
                        unsafe { self.get_reg_unchecked(src1) == self.get_reg_unchecked(src2) };
                    unsafe { self.set_reg_unchecked(dest, Value::Bool(are_equal)) };
                }
                OpCode::NotEqual(dest, src1, src2) => {
                    let are_not_equal =
                        unsafe { self.get_reg_unchecked(src1) != self.get_reg_unchecked(src2) };
                    unsafe { self.set_reg_unchecked(dest, Value::Bool(are_not_equal)) };
                }
                OpCode::GreaterThan(dest, src1, src2) => comparison_op!(self, dest, src1, src2, >)?,
                OpCode::LessThan(dest, src1, src2) => comparison_op!(self, dest, src1, src2, <)?,
                OpCode::GreaterThanOrEqual(dest, src1, src2) => {
                    comparison_op!(self, dest, src1, src2, >=)?
                }
                OpCode::LessThanOrEqual(dest, src1, src2) => {
                    comparison_op!(self, dest, src1, src2, <=)?
                }
                OpCode::JumpIfEq(r1, r2, addr) => {
                    if unsafe { self.get_reg_unchecked(r1) == self.get_reg_unchecked(r2) } {
                        self.ip = addr as usize;
                    }
                }
                OpCode::JumpIfNeq(r1, r2, addr) => {
                    if unsafe { self.get_reg_unchecked(r1) != self.get_reg_unchecked(r2) } {
                        self.ip = addr as usize;
                    }
                }
                OpCode::JumpIfGt(r1, r2, addr) => {
                    if let (Value::Number(v1), Value::Number(v2)) =
                        unsafe { (self.get_reg_unchecked(r1), self.get_reg_unchecked(r2)) }
                    {
                        if v1 > v2 {
                            self.ip = addr as usize;
                        }
                    }
                }
                OpCode::JumpIfGte(r1, r2, addr) => {
                    if let (Value::Number(v1), Value::Number(v2)) =
                        unsafe { (self.get_reg_unchecked(r1), self.get_reg_unchecked(r2)) }
                    {
                        if v1 >= v2 {
                            self.ip = addr as usize;
                        }
                    }
                }
                OpCode::JumpIfLt(r1, r2, addr) => {
                    if let (Value::Number(v1), Value::Number(v2)) =
                        unsafe { (self.get_reg_unchecked(r1), self.get_reg_unchecked(r2)) }
                    {
                        if v1 < v2 {
                            self.ip = addr as usize;
                        }
                    }
                }
                OpCode::JumpIfLte(r1, r2, addr) => {
                    if let (Value::Number(v1), Value::Number(v2)) =
                        unsafe { (self.get_reg_unchecked(r1), self.get_reg_unchecked(r2)) }
                    {
                        if v1 <= v2 {
                            self.ip = addr as usize;
                        }
                    }
                }
                OpCode::Jump(addr) => self.ip = addr as usize,
                OpCode::JumpIfFalse(reg, addr) => {
                    if let Value::Bool(false) = unsafe { self.get_reg_unchecked(reg) } {
                        self.ip = addr as usize;
                    }
                }
                OpCode::JumpIfTrue(reg, addr) => {
                    if let Value::Bool(true) = unsafe { self.get_reg_unchecked(reg) } {
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
