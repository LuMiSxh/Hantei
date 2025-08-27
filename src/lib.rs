//! # Hantei - Recipe Compilation and Evaluation Engine
//!
//! **Hantei** is a high-performance recipe compilation and evaluation engine that transforms
//! node-based decision trees into optimized Abstract Syntax Trees (ASTs). Built with Rust's
//! type safety and performance in mind, Hantei compiles UI-based recipes ahead of time for
//! lightning-fast runtime evaluation.
//!
//! ## Core Concepts
//!
//! - **Recipe**: A node-based decision tree defined in JSON format
//! - **Compilation**: Transform recipes into optimized ASTs with constant folding
//! - **Evaluation**: Execute compiled ASTs against runtime data
//! - **Quality**: Possible outcomes ranked by priority
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hantei::prelude::*;
//!
//! # fn main() -> Result<()> {
//! // Load and compile recipe
//! let recipe_json = std::fs::read_to_string("recipe.json")?;
//! let qualities_json = std::fs::read_to_string("qualities.json")?;
//!
//! let compiler = Compiler::new(&recipe_json, &qualities_json)?;
//! let (_logical_repr, compiled_paths) = compiler.compile()?;
//!
//! // Load sample data and evaluate
//! let sample_data = SampleData::from_file("sample_data.json")?;
//! let evaluator = Evaluator::new(compiled_paths);
//! let result = evaluator.eval(sample_data.static_data(), sample_data.dynamic_data())?;
//!
//! match result.quality_name {
//!     Some(name) => println!("Triggered: {} - {}", name, result.reason),
//!     None => println!("No quality triggered"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The compilation process follows these stages:
//!
//! 1. **Parse**: Load UI JSON format with nodes and edges
//! 2. **Compile**: Build ASTs for each quality path
//! 3. **Optimize**: Apply constant folding and logical simplification
//! 4. **Evaluate**: Execute optimized ASTs against runtime data
//!
//! ## Features
//!
//! - **High Performance**: Compile-time optimization eliminates runtime overhead
//! - **Type Safety**: Strong typing with comprehensive error handling
//! - **Cross-Product Evaluation**: Efficient handling of dynamic event combinations
//! - **Debug Output**: Detailed AST visualization and compilation traces
//! - **Memory Efficient**: Optimized data structures with minimal allocations

pub mod ast;
pub mod compiler;
pub mod data;
pub mod error;
pub mod evaluator;
pub mod trace;
pub mod ui;

// Prelude for convenient imports
pub mod prelude;

// Re-export commonly used types
pub use ast::{Expression, InputSource, Value};
pub use compiler::Compiler;
pub use data::SampleData;
pub use error::{CompileError, EvaluationError};
pub use evaluator::{EvaluationResult, Evaluator};
pub use trace::TraceFormatter;
pub use ui::{Quality, UiRecipe};
