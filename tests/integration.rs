//! End-to-end tests that verify the complete functionality works together.
mod common;
use ahash::AHashMap;
use common::*;
use hantei::backend::BackendChoice;
use hantei::prelude::*;

#[test]
fn test_full_workflow_simple() {
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();
    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().expect("Compilation failed");
    let evaluator = Evaluator::new(BackendChoice::Interpreter, artifacts).unwrap();

    let mut static_data = AHashMap::new();
    static_data.insert("Temperature".to_string(), 99.0);
    let dynamic_data = AHashMap::new();

    let result = evaluator
        .eval(&static_data, &dynamic_data)
        .expect("Evaluation failed");

    assert_eq!(result.quality_name.as_deref(), Some("Hot"));
    assert_eq!(result.quality_priority, Some(1));
}

#[test]
fn test_full_workflow_complex_dynamic() {
    let flow = create_complex_flow();
    let qualities = create_complex_qualities();
    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().expect("Compilation failed");
    let evaluator = Evaluator::new(BackendChoice::Interpreter, artifacts).unwrap();

    let static_data = create_sample_static_data();
    let dynamic_data = create_sample_dynamic_data();

    let result = evaluator
        .eval(&static_data, &dynamic_data)
        .expect("Evaluation failed");

    assert_eq!(result.quality_name.as_deref(), Some("Premium"));
    assert!(result.reason.contains("$Temperature (was 32.5) > 30"));
    assert!(result.reason.contains("$hole.Diameter (was 8.7) < 10"));
}
