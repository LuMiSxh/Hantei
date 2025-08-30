//! End-to-end tests that verify the complete functionality works together.
mod common;
use ahash::AHashMap;
use common::*;
use hantei::backend::BackendChoice;
use hantei::prelude::*;

#[test]
fn test_full_workflow_simple() {
    // 1. Define the recipe and qualities programmatically
    let flow = create_simple_flow();
    let qualities = create_simple_qualities();

    // 2. Build and run the compiler
    let compiler = Compiler::builder(flow, qualities).build();
    let compiled_paths = compiler.compile().expect("Compilation failed");

    // 3. Create the evaluator
    let evaluator = Evaluator::new(BackendChoice::Interpreter, compiled_paths).unwrap();

    // 4. Provide data and evaluate
    let mut static_data = AHashMap::new();
    static_data.insert("Temperature".to_string(), 99.0);
    let dynamic_data = AHashMap::new();

    let result = evaluator
        .eval(&static_data, &dynamic_data)
        .expect("Evaluation failed");

    // 5. Assert the result
    assert_eq!(result.quality_name.as_deref(), Some("Hot"));
    assert_eq!(result.quality_priority, Some(1));
}

#[test]
fn test_full_workflow_complex_dynamic() {
    // 1. Define recipe and qualities
    let flow = create_complex_flow();
    let qualities = create_complex_qualities();

    // 2. Compile
    let compiler = Compiler::builder(flow, qualities).build();
    let compiled_paths = compiler.compile().expect("Compilation failed");

    // 3. Evaluate
    let evaluator = Evaluator::new(BackendChoice::Interpreter, compiled_paths).unwrap();
    let static_data = create_sample_static_data(); // Temp = 32.5 (> 30)
    let dynamic_data = create_sample_dynamic_data(); // Hole Diameter = 8.7 (< 10)

    let result = evaluator
        .eval(&static_data, &dynamic_data)
        .expect("Evaluation failed");

    // 4. Assert
    assert_eq!(result.quality_name.as_deref(), Some("Premium"));
    assert!(result.reason.contains("$Temperature (was 32.5) > 30"));
    assert!(result.reason.contains("$hole.Diameter (was 8.7) < 10"));
}
