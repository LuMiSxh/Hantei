//! Tests for the compilation process, AST generation, and optimization.
mod common;
use common::*;
use hantei::prelude::*;

#[test]
fn test_compiler_builds_simple_flow() {
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();

    let compiler = Compiler::builder(flow, qualities).build();
    let compiled_paths = compiler.compile().expect("Failed to compile");

    assert_eq!(compiled_paths.len(), 1);
    let (prio, name, ast) = &compiled_paths[0];
    assert_eq!(*prio, 1);
    assert_eq!(name, "Hot");

    let ast_string = ast.to_string();
    assert!(ast_string.contains("$Temperature"));
    assert!(ast_string.contains(">"));
    assert!(ast_string.contains("25"));
}

#[test]
fn test_compiler_with_type_mapping() {
    let mut flow = create_simple_flow();
    // Modify the flow to use a custom node type name
    flow.nodes[1].operation_type = "MyGreaterThan".to_string();
    let qualities = create_simple_qualities();

    let compiler = Compiler::builder(flow, qualities)
        .with_type_mapping("MyGreaterThan", "gtNode")
        .build();

    let result = compiler.compile();
    assert!(
        result.is_ok(),
        "Compilation should succeed with type mapping"
    );
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_compiler_fails_on_unregistered_type() {
    let mut flow = create_simple_flow();
    flow.nodes[1].operation_type = "UnknownOperation".to_string(); // This doesn't exist
    let qualities = create_simple_qualities();

    let compiler = Compiler::builder(flow, qualities).build();
    let result = compiler.compile();
    assert!(result.is_err());

    match result.err().unwrap() {
        AstBuildError::InvalidNodeType { node_id, type_name } => {
            assert_eq!(node_id, "0002");
            assert_eq!(type_name, "UnknownOperation");
        }
        _ => panic!("Expected InvalidNodeType error"),
    }
}
