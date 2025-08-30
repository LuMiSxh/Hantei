//! # Hantei - Recipe Compilation and Evaluation Engine
//!
//! **Hantei** is a high-performance recipe compilation and evaluation engine that transforms
//! node-based decision trees into optimized Abstract Syntax Trees (ASTs). Built with Rust's
//! type safety and performance in mind, Hantei compiles recipes ahead of time for
//! lightning-fast runtime evaluation.
//!
//! ## Core Workflow
//!
//! The engine is designed to be format-agnostic. It operates on a canonical internal
//! model of a "flow definition." The primary workflow is:
//!
//! 1.  **Load Your Data**: Parse your custom recipe format (e.g., from JSON, YAML, etc.) into your own Rust structs.
//! 2.  **Convert to Hantei's Model**: Implement the `IntoFlow` trait for your structs to provide a translation layer into Hantei's `FlowDefinition`.
//! 3.  **Compile**: Use the `Compiler::builder` to create a compiler instance with the `FlowDefinition`. The compiler transforms this definition into highly optimized, executable ASTs.
//! 4.  **Evaluate**: Create an `Evaluator` with the compiled ASTs and run it repeatedly against different sets of runtime data.
//!
//! ## Quick Start
//!
//! The following example demonstrates the end-to-end process.
//!
//! ```rust,no_run
//! use hantei::prelude::*;
//! use hantei::recipe::{FlowDefinition, FlowNodeDefinition, FlowEdgeDefinition, Quality, IntoFlow};
//! use hantei::error::RecipeConversionError;
//! use hantei::backend::BackendChoice; // <-- FIX: Import BackendChoice
//! use ahash::AHashMap;
//!
//! // 1. Define structs that represent your custom recipe format.
//! struct MyNode {
//!     id: String,
//!     operation: String, // e.g., "GreaterThan"
//!     // ... other custom fields
//! }
//!
//! struct MyRecipe {
//!     nodes: Vec<MyNode>,
//!     // ... edges, etc.
//! }
//!
//! // 2. Implement the `IntoFlow` trait to convert your format to Hantei's.
//! impl IntoFlow for MyRecipe {
//!     fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> {
//!         // In a real implementation, you would loop through your nodes and edges
//!         // and map them to Hantei's FlowNodeDefinition and FlowEdgeDefinition.
//!         // This example returns a simple, hardcoded flow for demonstration.
//!         Ok(FlowDefinition {
//!             nodes: vec![
//!                 FlowNodeDefinition {
//!                     id: "0001".to_string(),
//!                     operation_type: "dynamicNode".to_string(),
//!                     input_type: None,
//!                     literal_values: None,
//!                     data_fields: Some(vec![DataFieldDefinition {
//!                         id: 0, name: "Temperature".to_string(), data_type: Some("number".to_string()),
//!                     }]),
//!                 },
//!                 FlowNodeDefinition {
//!                     id: "0002".to_string(),
//!                     operation_type: "gtNode".to_string(),
//!                     input_type: None,
//!                     literal_values: Some(vec![serde_json::Value::Null, serde_json::json!(25.0)]),
//!                     data_fields: None,
//!                 },
//!                 FlowNodeDefinition {
//!                     id: "0003".to_string(),
//!                     operation_type: "setQualityNode".to_string(),
//!                     input_type: None, literal_values: None, data_fields: None,
//!                 },
//!             ],
//!             edges: vec![
//!                 FlowEdgeDefinition {
//!                     source: "0001".to_string(), target: "0002".to_string(),
//!                     source_handle: "output-0".to_string(), target_handle: "input-0".to_string(),
//!                 },
//!                 FlowEdgeDefinition {
//!                     source: "0002".to_string(), target: "0003".to_string(),
//!                     source_handle: "output-0".to_string(), target_handle: "input-0".to_string(),
//!                 },
//!             ],
//!         })
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Assume `my_recipe` is loaded and parsed from a file
//!     let my_recipe = MyRecipe { nodes: vec![], /* ... */ };
//!     let qualities = vec![Quality { name: "Hot".to_string(), priority: 1 }];
//!
//!     // Convert your custom format into Hantei's canonical FlowDefinition
//!     let flow = my_recipe.into_flow()?;
//!
//!     // 3. Use the builder to create the compiler.
//!     let compiler = Compiler::builder(flow, qualities).build();
//!
//!     // Compile the flow into optimized ASTs
//!     println!("Compiling recipe...");
//!     let compiled_paths = compiler.compile()?;
//!     println!("Compilation successful!");
//!
//!     // 4. Create an evaluator, choosing a backend. Handle the Result.
//!     // --- FIX: Add BackendChoice and use `?` to handle the Result ---
//!     let evaluator = Evaluator::new(BackendChoice::Interpreter, compiled_paths)?;
//!
//!     let mut static_data = AHashMap::new();
//!     static_data.insert("Temperature".to_string(), 30.0); // This will trigger the rule
//!     let dynamic_data = AHashMap::new();
//!
//!     // Evaluate the data
//!     println!("Evaluating data...");
//!     let result = evaluator.eval(&static_data, &dynamic_data)?;
//!
//!     // Print the result
//!     if let Some(name) = result.quality_name {
//!         println!("-> Triggered Quality: {} (Priority: {})", name, result.quality_priority.unwrap());
//!         println!("-> Reason: {}", result.reason);
//!     } else {
//!         println!("-> No quality was triggered.");
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod ast;
pub mod backend;
pub mod bytecode;
pub mod compiler;
pub mod data;
pub mod error;
pub mod evaluator;
pub mod interpreter;
pub mod prelude;
pub mod recipe;
pub mod trace;

#[cfg(feature = "python-bindings")]
mod python;
