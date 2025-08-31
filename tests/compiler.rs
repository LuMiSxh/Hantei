//! Tests for the compilation process, AST generation, and optimization.
mod common;
use common::*;
use hantei::prelude::*;

#[test]
fn test_compiler_builds_simple_flow() {
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();

    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().expect("Failed to compile");

    assert_eq!(artifacts.len(), 1);
    let first_artifact = &artifacts[0];
    assert_eq!(first_artifact.priority, 1);
    assert_eq!(first_artifact.name, "Hot");

    // Check that the string interning worked
    assert_eq!(first_artifact.static_map.len(), 1);
    assert!(first_artifact.static_map.contains_key("Temperature"));
}

#[test]
fn test_compiler_with_type_mapping() {
    let mut flow = create_simple_flow();
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
    flow.nodes[1].operation_type = "UnknownOperation".to_string();
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
