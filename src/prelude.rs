//! Prelude module for convenient imports
//!
//! This module re-exports the most commonly used types and traits from the hantei crate.
//! Import this module to get access to the core functionality without having to import
//! each type individually.
//!
//! # Example
//!
//! ```rust,no_run
//! // Use the prelude to get easy access to all the core types.
//! use hantei::prelude::*;
//!
//! # fn run_example() -> Result<()> {
//! // Load and compile recipe
//! let recipe_json = std::fs::read_to_string("path/to/recipe.json")?;
//! let qualities_json = std::fs::read_to_string("path/to/qualities.json")?;
//!
//! let compiler = Compiler::new(&recipe_json, &qualities_json)?;
//! let (_logical_repr, compiled_paths) = compiler.compile(false)?;
//!
//! // Load sample data and evaluate
//! let sample_data = SampleData::from_file("path/to/data.json")?;
//! let evaluator = Evaluator::new(compiled_paths);
//! let result = evaluator.eval(sample_data.static_data(), sample_data.dynamic_data())?;
//!
//! println!("Evaluation Result: {:?}", result);
//! # Ok(())
//! # }
//! ```

// Core compilation and evaluation
pub use crate::compiler::Compiler;
pub use crate::evaluator::{EvaluationResult, Evaluator};

// AST and expression types
pub use crate::ast::{EvaluationTrace, Expression, InputSource, Value};

// Data structures
pub use crate::data::SampleData;
pub use crate::ui::{Quality, UiRecipe};

// Error types
pub use crate::error::{CompileError, EvaluationError};

// Trace formatting
pub use crate::trace::TraceFormatter;

// Standard library re-exports commonly used with this crate
pub use std::collections::HashMap;
pub use std::path::Path;

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
