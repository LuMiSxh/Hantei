//! # Hantei - Recipe Compilation and Evaluation Engine
//!
//! ```rust,no_run
//! use hantei::prelude::*;
//! use hantei::recipe::{FlowDefinition, Quality, IntoFlow};
//! use hantei::error::RecipeConversionError;
//! use hantei::backend::BackendChoice;
//! use ahash::AHashMap;
//!
//! // Assume MyRecipe and its IntoFlow impl exist...
//! # struct MyRecipe;
//! # impl IntoFlow for MyRecipe { fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> { Ok(FlowDefinition::default()) } }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let my_recipe = MyRecipe { /* ... */ };
//!     let qualities = vec![Quality { name: "Hot".to_string(), priority: 1 }];
//!
//!     // Convert your custom format into Hantei's canonical FlowDefinition
//!     let flow = my_recipe.into_flow()?;
//!
//!     // 3. Use the builder to create the compiler.
//!     let compiler = Compiler::builder(flow, qualities).build();
//!
//!     // --- MODIFIED: The compile step now produces artifacts ---
//!     println!("Compiling recipe...");
//!     let compiled_artifacts = compiler.compile()?;
//!     println!("Compilation successful!");
//!
//!     // 4. Create an evaluator from the artifacts.
//!     let evaluator = Evaluator::new(BackendChoice::Bytecode, compiled_artifacts)?;
//!
//!     let static_data = AHashMap::new();
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
