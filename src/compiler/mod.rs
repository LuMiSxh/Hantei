use crate::ast::{Expression, InputId, InputSource, Value};
use crate::error::AstBuildError;
use crate::recipe::{FlowDefinition, Quality};
use ahash::AHashMap;

#[cfg(feature = "debug-tools")]
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

pub struct CompilationArtifacts {
    pub priority: i32,
    pub name: String,
    pub ast: Expression,
    pub definitions: AHashMap<u64, Expression>,
    pub static_map: AHashMap<String, InputId>,
    pub dynamic_map: AHashMap<String, InputId>,
}

pub struct Compiler {
    flow: FlowDefinition,
    qualities: Vec<Quality>,
    registry: AHashMap<String, Box<dyn NodeParser>>,
    ast_cache: AHashMap<String, Expression>,
    static_map: AHashMap<String, InputId>,
    dynamic_map: AHashMap<String, InputId>,
    next_static_id: InputId,
    next_dynamic_id: InputId,
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
            static_map: AHashMap::new(),
            dynamic_map: AHashMap::new(),
            next_static_id: 0,
            next_dynamic_id: 0,
        }
    }
}

impl Compiler {
    pub fn builder(flow: FlowDefinition, qualities: Vec<Quality>) -> CompilerBuilder {
        CompilerBuilder::new(flow, qualities)
    }

    // String interning methods
    fn get_static_id(&mut self, name: &str) -> InputId {
        *self.static_map.entry(name.to_string()).or_insert_with(|| {
            let id = self.next_static_id;
            self.next_static_id += 1;
            id
        })
    }

    fn get_dynamic_id(&mut self, event: &str, field: &str) -> InputId {
        let key = format!("{}.{}", event, field);
        *self.dynamic_map.entry(key).or_insert_with(|| {
            let id = self.next_dynamic_id;
            self.next_dynamic_id += 1;
            id
        })
    }

    /// Recursively transforms an AST with string-based inputs into one with ID-based inputs.
    fn intern_ast_inputs(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::Input(source) => match source {
                InputSource::StaticName { name } => {
                    let id = self.get_static_id(&name);
                    Expression::Input(InputSource::Static { id })
                }
                InputSource::DynamicName { event, field } => {
                    let id = self.get_dynamic_id(&event, &field);
                    Expression::Input(InputSource::Dynamic { id })
                }
                // Already interned
                other => Expression::Input(other),
            },
            Expression::Sum(l, r) => Expression::Sum(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Subtract(l, r) => Expression::Subtract(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Multiply(l, r) => Expression::Multiply(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Divide(l, r) => Expression::Divide(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Abs(v) => Expression::Abs(Box::new(self.intern_ast_inputs(*v))),
            Expression::Not(v) => Expression::Not(Box::new(self.intern_ast_inputs(*v))),
            Expression::And(l, r) => Expression::And(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Or(l, r) => Expression::Or(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Xor(l, r) => Expression::Xor(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::Equal(l, r) => Expression::Equal(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::NotEqual(l, r) => Expression::NotEqual(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::GreaterThan(l, r) => Expression::GreaterThan(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::GreaterThanOrEqual(l, r) => Expression::GreaterThanOrEqual(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::SmallerThan(l, r) => Expression::SmallerThan(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            Expression::SmallerThanOrEqual(l, r) => Expression::SmallerThanOrEqual(
                Box::new(self.intern_ast_inputs(*l)),
                Box::new(self.intern_ast_inputs(*r)),
            ),
            // Leaf nodes that don't need changes
            other => other,
        }
    }

    pub fn compile(mut self) -> Result<Vec<CompilationArtifacts>, AstBuildError> {
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

        let mut quality_artifacts = Vec::new();

        // Clone the qualities to avoid borrowing issues during iteration
        let qualities = self.qualities.clone();

        for (index, quality) in qualities.iter().enumerate() {
            if let Some(naive_ast) = naive_ast_map.get(&(index as u32)) {
                if let Expression::Literal(Value::Null) = naive_ast {
                    continue;
                }

                // 1. Intern the strings in the naive AST to get an ID-based AST
                let interned_ast = self.intern_ast_inputs(naive_ast.clone());

                // 2. Optimize the ID-based AST
                let mut optimizer = AstOptimizer::new();
                let optimized_ast = optimizer.optimize(interned_ast);
                let definitions = optimizer.definitions;

                #[cfg(feature = "debug-tools")]
                {
                    // Create reverse maps for debugging output
                    let static_rev_map: AHashMap<InputId, String> = self
                        .static_map
                        .iter()
                        .map(|(k, v)| (*v, k.clone()))
                        .collect();
                    let dynamic_rev_map: AHashMap<InputId, String> = self
                        .dynamic_map
                        .iter()
                        .map(|(k, v)| (*v, k.clone()))
                        .collect();

                    let sanitized_name = self.sanitize_filename(&quality.name);
                    let naive_display = DisplayExpression {
                        expr: naive_ast,
                        definitions: &AHashMap::new(),
                        static_map: &static_rev_map,
                        dynamic_map: &dynamic_rev_map,
                    };
                    self.write_debug_file(
                        &format!("tmp/quality_{}_naive_ast.txt", &sanitized_name),
                        &naive_display.to_string(),
                    )?;
                    let optimized_display = DisplayExpression {
                        expr: &optimized_ast,
                        definitions: &definitions,
                        static_map: &static_rev_map,
                        dynamic_map: &dynamic_rev_map,
                    };
                    self.write_debug_file(
                        &format!("tmp/quality_{}_optimized_ast.txt", &sanitized_name),
                        &optimized_display.to_string(),
                    )?;

                    match bytecode_compiler::compile_to_program(
                        &optimized_ast,
                        &definitions,
                        &self.static_map,
                        &self.dynamic_map,
                    ) {
                        Ok(program) => {
                            let viz = bytecode_visualizer::visualize_program(
                                &program,
                                &quality.name,
                                &static_rev_map,
                                &dynamic_rev_map,
                            );
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

                quality_artifacts.push(CompilationArtifacts {
                    priority: quality.priority,
                    name: quality.name.clone(),
                    ast: optimized_ast,
                    definitions,
                    static_map: self.static_map.clone(),
                    dynamic_map: self.dynamic_map.clone(),
                });
            }
        }

        quality_artifacts.sort_by_key(|a| a.priority);
        Ok(quality_artifacts)
    }

    #[cfg(feature = "debug-tools")]
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }
    #[cfg(feature = "debug-tools")]
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
