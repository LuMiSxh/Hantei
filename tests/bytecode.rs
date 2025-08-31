//! Unit tests for the register-based bytecode compiler and virtual machine.
mod common;
use ahash::AHashMap;
use hantei::ast::{Expression, InputSource, Value};
use hantei::bytecode::compiler::compile_to_program;
use hantei::bytecode::vm::Vm;

#[test]
fn test_vm_simple_arithmetic() {
    let ast = Expression::Subtract(
        Box::new(Expression::Literal(Value::Number(10.0))),
        Box::new(Expression::Literal(Value::Number(5.0))),
    );

    let program =
        compile_to_program(&ast, &AHashMap::new(), &AHashMap::new(), &AHashMap::new()).unwrap();

    let static_data_vec = Vec::new();
    let dynamic_data_vec = Vec::new();
    let mut vm = Vm::new(&program, &static_data_vec, &dynamic_data_vec);
    let result = vm.run().unwrap();

    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_vm_data_loading() {
    let ast = Expression::Sum(
        Box::new(Expression::Input(InputSource::Static { id: 0 })),
        Box::new(Expression::Input(InputSource::Dynamic { id: 0 })),
    );

    let mut static_map = AHashMap::new();
    static_map.insert("Temp".to_string(), 0);
    let mut dynamic_map = AHashMap::new();
    dynamic_map.insert("hole.Diameter".to_string(), 0);

    let program = compile_to_program(&ast, &AHashMap::new(), &static_map, &dynamic_map).unwrap();

    let static_data = vec![Value::Number(100.0)];
    let dynamic_data = vec![Value::Number(25.0)];

    let mut vm = Vm::new(&program, &static_data, &dynamic_data);
    let result = vm.run().unwrap();
    assert_eq!(result, Value::Number(125.0));
}
