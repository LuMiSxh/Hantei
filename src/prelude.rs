//! # Hantei Prelude
//!
//! This module re-exports the most commonly used types and traits from the `hantei`
//! crate for convenience.
//!
//! By importing this prelude, you get easy access to the core components needed to
//! build a compiler, create an evaluator, and handle results, without needing

//! to import each type individually.
//!
//! ## Example
//!
//! ```rust,no_run
//! // Use the prelude to bring all core types into scope.
//! use hantei::prelude::*;
//!
//! fn run_example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Now you can directly use types like `Compiler`, `Evaluator`, `FlowDefinition`, etc.
//!     // let flow: FlowDefinition = ...;
//!     // let qualities: Vec<Quality> = ...;
//!     // let compiler = Compiler::builder(flow, qualities).build();
//!     Ok(())
//! }
//! ```

// Core compilation and evaluation
pub use crate::compiler::{Compiler, CompilerBuilder};
pub use crate::evaluator::Evaluator;
pub use crate::interpreter::EvaluationResult;

// AST and expression types
pub use crate::ast::{EvaluationTrace, Expression, InputSource, Value};

// Recipe data structures and traits
pub use crate::recipe::{
    DataFieldDefinition, FlowDefinition, FlowEdgeDefinition, FlowNodeDefinition, IntoFlow, Quality,
};

// Runtime data model
pub use crate::data::SampleData;

// Error types
pub use crate::error::{
    AstBuildError, BackendError, EvaluationError, RecipeConversionError, VmError,
};

// Trace formatting
pub use crate::trace::TraceFormatter;

// Standard library re-exports
pub use std::path::Path;
