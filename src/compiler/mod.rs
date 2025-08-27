use crate::ast::{Expression, Value};
use crate::error::CompileError;
use crate::recipe::{FlowDefinition, Quality};
use std::collections::HashMap;
#[cfg(feature = "hantei-cli")]
use std::fs;

mod builder;
mod optimizer;
pub mod parsing;

use builder::AstBuilder;
use optimizer::AstOptimizer;
use parsing::*;

/// Compiles a `FlowDefinition` into optimized, executable Abstract Syntax Trees (ASTs).
///
/// This struct is the main entry point for the compilation phase. It takes a canonical
/// `FlowDefinition` and a list of `Quality` outcomes and transforms them into a set of
/// highly optimized `Expression` trees, one for each quality path.
///
/// It is recommended to create a `Compiler` using the [`Compiler::builder`] method to allow
/// for future customization.
pub struct Compiler {
    flow: FlowDefinition,
    qualities: Vec<Quality>,
    registry: HashMap<String, Box<dyn NodeParser>>,
    // Cache to avoid re-computing ASTs for the same nodes
    ast_cache: HashMap<String, Expression>,
}

/// A builder for creating a `Compiler` with custom configurations.
///
/// This builder provides a fluent API for setting up the compiler. It allows for advanced
/// features like mapping custom node type names or registering entirely new node parsers.
pub struct CompilerBuilder {
    flow: FlowDefinition,
    qualities: Vec<Quality>,
    registry: HashMap<String, Box<dyn NodeParser>>,
}

impl CompilerBuilder {
    /// Creates a new builder with the default set of built-in node parsers.
    ///
    /// # Arguments
    /// * `flow`: The canonical `FlowDefinition` representing the logic to be compiled.
    /// * `qualities`: A `Vec<Quality>` defining the possible outcomes.
    pub fn new(flow: FlowDefinition, qualities: Vec<Quality>) -> Self {
        let mut registry: HashMap<String, Box<dyn NodeParser>> = HashMap::new();
        // Register all built-in node parsers
        register_default_parsers(&mut registry);
        Self {
            flow,
            qualities,
            registry,
        }
    }

    /// Maps a custom, user-defined node type name to one of Hantei's built-in parsers.
    ///
    /// This is the primary mechanism for supporting recipe formats that use different
    /// names for standard operations.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use hantei::prelude::*;
    /// # let flow = FlowDefinition::default();
    /// # let qualities = vec![];
    /// // If your recipe JSON uses "CompareGreaterThan" instead of "gtNode":
    /// let compiler = Compiler::builder(flow, qualities)
    ///     .with_type_mapping("CompareGreaterThan", "gtNode")
    ///     .build();
    /// ```
    pub fn with_type_mapping(mut self, user_type_name: &str, hantei_type_name: &str) -> Self {
        if let Some(parser) = create_parser_by_name(hantei_type_name) {
            self.registry.insert(user_type_name.to_string(), parser);
        }
        self
    }

    /// Registers a new, custom `NodeParser` implementation.
    ///
    /// This allows for extending Hantei with entirely new operations without modifying
    /// the core compiler. The provided parser must implement the [`NodeParser`] trait.
    pub fn with_custom_parser(mut self, parser: Box<dyn NodeParser>) -> Self {
        self.registry.insert(parser.node_type().to_string(), parser);
        self
    }

    /// Consumes the builder and constructs the final `Compiler` instance.
    pub fn build(self) -> Compiler {
        Compiler {
            flow: self.flow,
            qualities: self.qualities,
            registry: self.registry,
            ast_cache: HashMap::new(),
        }
    }
}

impl Compiler {
    /// Creates a `CompilerBuilder` to configure and create a compiler.
    /// This is the recommended way to instantiate a `Compiler`.
    pub fn builder(flow: FlowDefinition, qualities: Vec<Quality>) -> CompilerBuilder {
        CompilerBuilder::new(flow, qualities)
    }

    /// Compiles the flow definition into a `Vec` of `(priority, name, Expression)`.
    ///
    /// This method consumes the compiler and executes the full compilation pipeline:
    /// 1. Builds a naive AST for each quality path.
    /// 2. Applies optimization passes (e.g., constant folding) to each AST.
    /// 3. Returns the final, optimized ASTs, sorted by quality priority.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the vector of compiled paths or a `CompileError`.
    pub fn compile(mut self) -> Result<Vec<(i32, String, Expression)>, CompileError> {
        let quality_node_id = self
            .flow
            .nodes
            .iter()
            .find(|n| n.operation_type == "setQualityNode")
            .ok_or_else(|| CompileError::InvalidNodeType {
                node_id: "N/A".to_string(),
                type_name: "setQualityNode not found".to_string(),
            })?
            .id
            .clone();

        let mut ast_builder = AstBuilder::new(&self.flow, &self.registry, &mut self.ast_cache);
        let naive_ast_map = ast_builder.build_asts_for_node(&quality_node_id)?;

        let mut quality_asts = Vec::new();
        let optimizer = AstOptimizer::new();

        for (index, quality) in self.qualities.iter().enumerate() {
            if let Some(naive_ast) = naive_ast_map.get(&(index as u32)) {
                if let Expression::Literal(Value::Null) = naive_ast {
                    continue; // Skip empty/unconnected quality slots
                }

                let optimized_ast = optimizer.optimize(naive_ast.clone());

                #[cfg(feature = "hantei-cli")]
                {
                    let sanitized_name = self.sanitize_filename(&quality.name);
                    self.write_debug_file(
                        &format!("tmp/quality_{}_naive_ast.txt", &sanitized_name),
                        &naive_ast.to_string(),
                    )?;
                    self.write_debug_file(
                        &format!("tmp/quality_{}_optimized_ast.txt", &sanitized_name),
                        &optimized_ast.to_string(),
                    )?;
                }
                quality_asts.push((quality.priority, quality.name.clone(), optimized_ast));
            }
        }

        quality_asts.sort_by_key(|(p, _, _)| *p);
        Ok(quality_asts)
    }

    #[cfg(feature = "hantei-cli")]
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }

    #[cfg(feature = "hantei-cli")]
    fn write_debug_file(&self, path: &str, content: &str) -> Result<(), CompileError> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CompileError::JsonParseError(format!("Failed to create debug directory: {}", e))
            })?;
        }
        fs::write(path, content)
            .map_err(|e| CompileError::JsonParseError(format!("Failed to write debug file: {}", e)))
    }
}
