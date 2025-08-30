//! Unit tests for the bytecode compiler and virtual machine.
mod common;
use ahash::AHashMap;
use hantei::ast::{Expression, InputSource, Value};
use hantei::bytecode::compiler::compile_ast;
use hantei::bytecode::opcode::OpCode;
use hantei::bytecode::vm::Vm;
use hantei::error::VmError;

#[test]
fn test_compile_literals_and_inputs() {
    let expr = Expression::Input(InputSource::Static {
        name: "Temp".to_string(),
    });
    let bytecode = compile_ast(&expr).unwrap();
    assert_eq!(
        bytecode,
        vec![OpCode::LoadStatic("Temp".to_string()), OpCode::Return]
    );

    let expr_lit = Expression::Literal(Value::Number(123.0));
    let bytecode_lit = compile_ast(&expr_lit).unwrap();
    assert_eq!(
        bytecode_lit,
        vec![OpCode::Push(Value::Number(123.0)), OpCode::Return]
    );
}

#[test]
fn test_compile_simple_comparison() {
    let expr = Expression::GreaterThan(
        Box::new(Expression::Input(InputSource::Static {
            name: "Temp".to_string(),
        })),
        Box::new(Expression::Literal(Value::Number(25.0))),
    );
    let bytecode = compile_ast(&expr).unwrap();
    assert_eq!(
        bytecode,
        vec![
            OpCode::LoadStatic("Temp".to_string()),
            OpCode::Push(Value::Number(25.0)),
            OpCode::GreaterThan,
            OpCode::Return,
        ]
    );
}

#[test]
fn test_compile_and_short_circuit() {
    let expr = Expression::And(
        Box::new(Expression::Literal(Value::Bool(false))),
        Box::new(Expression::Literal(Value::Number(1.0))), // This part should be skipped
    );
    let bytecode = compile_ast(&expr).unwrap();
    assert_eq!(
        bytecode,
        vec![
            OpCode::Push(Value::Bool(false)),
            OpCode::JumpIfFalse(4), // Jumps to the Return instruction
            OpCode::Pop,
            OpCode::Push(Value::Number(1.0)),
            // Jump target is here
            OpCode::Return,
        ]
    );
}

#[test]
fn test_vm_simple_arithmetic() {
    let bytecode = vec![
        OpCode::Push(Value::Number(10.0)),
        OpCode::Push(Value::Number(5.0)),
        OpCode::Subtract,
        OpCode::Return,
    ];
    let static_data = AHashMap::new();
    let dynamic_data = AHashMap::new();
    let mut vm = Vm::new(&bytecode, &static_data, &dynamic_data);
    let result = vm.run().unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_vm_conditional_jump() {
    // Logic: if false, jump to instruction 4 (push 99), else continue
    let bytecode = vec![
        OpCode::Push(Value::Bool(false)),
        OpCode::JumpIfFalse(4),
        OpCode::Push(Value::Number(1.0)), // This should be skipped
        OpCode::Return,
        OpCode::Push(Value::Number(99.0)), // Jump lands here
        OpCode::Return,
    ];
    let static_data = AHashMap::new();
    let dynamic_data = AHashMap::new();
    let mut vm = Vm::new(&bytecode, &static_data, &dynamic_data);
    let result = vm.run().unwrap();
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_vm_data_loading() {
    let bytecode = vec![
        OpCode::LoadStatic("Temp".to_string()),
        OpCode::LoadDynamic("hole".to_string(), "Diameter".to_string()),
        OpCode::Add,
        OpCode::Return,
    ];

    let static_data = AHashMap::from([("Temp".to_string(), 100.0)]);
    let hole_instance = AHashMap::from([("Diameter".to_string(), 25.0)]);
    let dynamic_context = AHashMap::from([("hole".to_string(), &hole_instance)]);

    let mut vm = Vm::new(&bytecode, &static_data, &dynamic_context);
    let result = vm.run().unwrap();
    assert_eq!(result, Value::Number(125.0));
}

#[test]
fn test_vm_stack_underflow() {
    let bytecode = vec![OpCode::Add, OpCode::Return]; // Add needs two values, stack is empty
    let static_data = AHashMap::new();
    let dynamic_data = AHashMap::new();
    let mut vm = Vm::new(&bytecode, &static_data, &dynamic_data);
    let result = vm.run();
    assert_eq!(result, Err(VmError::StackUnderflow));
}
