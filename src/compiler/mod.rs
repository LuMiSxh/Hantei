use crate::ast::{Expression, Value};
use crate::error::AstBuildError;
use crate::recipe::{FlowDefinition, Quality};
use ahash::AHashMap;
#[cfg(all(feature = "hantei-cli", debug_assertions))]
use {
    crate::ast::DisplayExpression,
    crate::bytecode::{compiler as bytecode_compiler, visualizer as bytecode_visualizer},
    std::fs,
};

mod builder;
mod optimizer;
pub mod parsing;

use builder::AstBuilder;
use optimizer::AstOptimizer;
use parsing::*;

pub struct Compiler {
    flow: FlowDefinition,
    qualities: Vec<Quality>,
    registry: AHashMap<String, Box<dyn NodeParser>>,
    ast_cache: AHashMap<String, Expression>,
}

pub struct CompilerBuilder {
    flow: FlowDefinition,
    qualities: Vec<Quality>,
    registry: AHashMap<String, Box<dyn NodeParser>>,
}

impl CompilerBuilder {
    pub fn new(flow: FlowDefinition, qualities: Vec<Quality>) -> Self {
        let mut registry: AHashMap<String, Box<dyn NodeParser>> = AHashMap::new();
        register_default_parsers(&mut registry);
        Self {
            flow,
            qualities,
            registry,
        }
    }
    pub fn with_type_mapping(mut self, user_type_name: &str, hantei_type_name: &str) -> Self {
        if let Some(parser) = create_parser_by_name(hantei_type_name) {
            self.registry.insert(user_type_name.to_string(), parser);
        }
        self
    }
    pub fn with_custom_parser(mut self, parser: Box<dyn NodeParser>) -> Self {
        self.registry.insert(parser.node_type().to_string(), parser);
        self
    }
    pub fn build(self) -> Compiler {
        Compiler {
            flow: self.flow,
            qualities: self.qualities,
            registry: self.registry,
            ast_cache: AHashMap::new(),
        }
    }
}

impl Compiler {
    pub fn builder(flow: FlowDefinition, qualities: Vec<Quality>) -> CompilerBuilder {
        CompilerBuilder::new(flow, qualities)
    }

    pub fn compile(
        mut self,
    ) -> Result<Vec<(i32, String, Expression, AHashMap<u64, Expression>)>, AstBuildError> {
        let quality_node_id = self
            .flow
            .nodes
            .iter()
            .find(|n| n.operation_type == "setQualityNode")
            .ok_or_else(|| AstBuildError::InvalidNodeType {
                node_id: "N/A".to_string(),
                type_name: "setQualityNode not found".to_string(),
            })?
            .id
            .clone();

        let mut ast_builder = AstBuilder::new(&self.flow, &self.registry, &mut self.ast_cache);
        let naive_ast_map = ast_builder.build_asts_for_node(&quality_node_id)?;

        let mut quality_asts = Vec::new();
        let mut optimizer = AstOptimizer::new();

        for (index, quality) in self.qualities.iter().enumerate() {
            if let Some(naive_ast) = naive_ast_map.get(&(index as u32)) {
                if let Expression::Literal(Value::Null) = naive_ast {
                    continue;
                }

                let optimized_ast = optimizer.optimize(naive_ast.clone());

                // The definitions map will be built up across all qualities
                let definitions = optimizer.definitions.clone();

                #[cfg(all(feature = "hantei-cli", debug_assertions))]
                {
                    let sanitized_name = self.sanitize_filename(&quality.name);
                    let naive_display = DisplayExpression {
                        expr: naive_ast,
                        definitions: &AHashMap::new(),
                    };
                    self.write_debug_file(
                        &format!("tmp/quality_{}_naive_ast.txt", &sanitized_name),
                        &naive_display.to_string(),
                    )?;

                    let optimized_display = DisplayExpression {
                        expr: &optimized_ast,
                        definitions: &definitions,
                    };
                    self.write_debug_file(
                        &format!("tmp/quality_{}_optimized_ast.txt", &sanitized_name),
                        &optimized_display.to_string(),
                    )?;

                    match bytecode_compiler::compile_to_program(&optimized_ast, &definitions) {
                        Ok(program) => {
                            let viz =
                                bytecode_visualizer::visualize_program(&program, &quality.name);
                            self.write_debug_file(
                                &format!("tmp/quality_{}_bytecode.txt", &sanitized_name),
                                &viz,
                            )?;
                        }
                        Err(e) => eprintln!(
                            "Warning: Could not compile bytecode for debug file for quality '{}': {}",
                            quality.name, e
                        ),
                    }
                }
                quality_asts.push((
                    quality.priority,
                    quality.name.clone(),
                    optimized_ast,
                    definitions,
                ));
            }
        }

        quality_asts.sort_by_key(|(p, _, _, _)| *p);
        Ok(quality_asts)
    }

    #[cfg(all(feature = "hantei-cli", debug_assertions))]
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }

    #[cfg(all(feature = "hantei-cli", debug_assertions))]
    fn write_debug_file(&self, path: &str, content: &str) -> Result<(), AstBuildError> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AstBuildError::JsonParseError(format!("Failed to create debug directory: {}", e))
            })?;
        }
        fs::write(path, content).map_err(|e| {
            AstBuildError::JsonParseError(format!("Failed to write debug file: {}", e))
        })
    }
}
