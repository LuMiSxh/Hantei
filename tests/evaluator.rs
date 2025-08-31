//! Tests for the evaluation engine and runtime data handling.
mod common;
use ahash::AHashMap;
use common::*;
use hantei::backend::BackendChoice;
use hantei::prelude::*;

#[test]
fn test_static_evaluation_trigger() {
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();
    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().unwrap();
    let evaluator = Evaluator::new(BackendChoice::Interpreter, artifacts).unwrap();

    let mut static_data = AHashMap::new();
    static_data.insert("Temperature".to_string(), 30.0);
    let dynamic_data = AHashMap::new();

    let result = evaluator.eval(&static_data, &dynamic_data).unwrap();
    assert_eq!(result.quality_name.as_deref(), Some("Hot"));
    assert!(result.reason.contains("$Temperature (was 30) > 25"));
}

#[test]
fn test_static_evaluation_no_trigger() {
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();
    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().unwrap();
    let evaluator = Evaluator::new(BackendChoice::Interpreter, artifacts).unwrap();

    let mut static_data = AHashMap::new();
    static_data.insert("Temperature".to_string(), 20.0);
    let dynamic_data = AHashMap::new();

    let result = evaluator.eval(&static_data, &dynamic_data).unwrap();
    assert!(result.quality_name.is_none());
}

#[test]
fn test_dynamic_cross_product_evaluation() {
    let flow = create_complex_flow();
    let qualities = create_complex_qualities();
    let compiler = Compiler::builder(flow, qualities).build();
    let artifacts = compiler.compile().unwrap();
    let evaluator = Evaluator::new(BackendChoice::Interpreter, artifacts).unwrap();

    let mut static_data = AHashMap::new();
    static_data.insert("Temperature".to_string(), 35.0);

    let mut dynamic_data = AHashMap::new();
    let mut hole_events = Vec::new();
    hole_events.push(AHashMap::from([("Diameter".to_string(), 12.0)]));
    hole_events.push(AHashMap::from([("Diameter".to_string(), 8.0)]));
    dynamic_data.insert("hole".to_string(), hole_events);

    let result = evaluator.eval(&static_data, &dynamic_data).unwrap();
    assert_eq!(result.quality_name.as_deref(), Some("Premium"));
    assert!(result.reason.contains("$hole.Diameter (was 8)"));
}
