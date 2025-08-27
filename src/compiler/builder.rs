use crate::ast::{Expression, InputSource, Value};
use crate::compiler::parsing::NodeParser;
use crate::error::CompileError;
use crate::recipe::{FlowDefinition, FlowNodeDefinition};
use std::collections::HashMap;

/// Responsible for building the initial, unoptimized AST from a `FlowDefinition`.
pub(super) struct AstBuilder<'a> {
    flow: &'a FlowDefinition,
    registry: &'a HashMap<String, Box<dyn NodeParser>>,
    ast_cache: &'a mut HashMap<String, Expression>,
    connections: HashMap<String, HashMap<u32, Vec<(String, u32)>>>,
}

impl<'a> AstBuilder<'a> {
    pub(super) fn new(
        flow: &'a FlowDefinition,
        registry: &'a HashMap<String, Box<dyn NodeParser>>,
        ast_cache: &'a mut HashMap<String, Expression>,
    ) -> Self {
        let mut connections: HashMap<String, HashMap<u32, Vec<(String, u32)>>> = HashMap::new();
        for edge in &flow.edges {
            let target_handle_idx = Self::parse_handle_index(&edge.target_handle);
            let source_handle_idx = Self::parse_handle_index(&edge.source_handle);
            connections
                .entry(edge.target.clone())
                .or_default()
                .entry(target_handle_idx)
                .or_default()
                .push((edge.source.clone(), source_handle_idx));
        }

        Self {
            flow,
            registry,
            ast_cache,
            connections,
        }
    }

    /// Builds all ASTs that feed into a specific target node.
    /// Returns a map of `target_handle_index -> combined_ast`.
    pub(super) fn build_asts_for_node(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<u32, Expression>, CompileError> {
        let node = self.find_node(node_id, "N/A")?;
        let mut expressions = self.gather_connected_inputs(node_id)?;

        if let Some(values) = &node.literal_values {
            for (i, val) in values.iter().enumerate() {
                expressions.entry(i as u32).or_insert_with(|| {
                    if let Some(num) = val.as_f64() {
                        Expression::Literal(Value::Number(num))
                    } else if let Some(b) = val.as_bool() {
                        Expression::Literal(Value::Bool(b))
                    } else {
                        Expression::Literal(Value::Null)
                    }
                });
            }
        }

        Ok(expressions)
    }

    /// Recursively builds the AST for a single node, handling caching.
    fn build_ast(&mut self, node_id: &str, source_id: &str) -> Result<Expression, CompileError> {
        if let Some(cached) = self.ast_cache.get(node_id) {
            return Ok(cached.clone());
        }

        let node = self.find_node(node_id, source_id)?;
        let expressions_map = self.build_asts_for_node(node_id)?;

        let mut sorted_expressions: Vec<_> = expressions_map.into_iter().collect();
        sorted_expressions.sort_by_key(|(idx, _)| *idx);
        let inputs: Vec<Expression> = sorted_expressions
            .into_iter()
            .map(|(_, expr)| expr)
            .collect();

        let parser = self.registry.get(&node.operation_type).ok_or_else(|| {
            CompileError::InvalidNodeType {
                node_id: node.id.clone(),
                type_name: node.operation_type.clone(),
            }
        })?;

        let expression = parser.parse(node, inputs)?;
        self.ast_cache
            .insert(node_id.to_string(), expression.clone());
        Ok(expression)
    }

    /// Gathers all incoming connected expressions for a node.
    fn gather_connected_inputs(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<u32, Expression>, CompileError> {
        let mut expressions: HashMap<u32, Expression> = HashMap::new();

        // **FIX:** Clone the connection data to iterate over, releasing the borrow on `self`.
        let connections_to_process: Vec<(u32, Vec<(String, u32)>)> = self
            .connections
            .get(node_id)
            .map(|conn_map| conn_map.iter().map(|(k, v)| (*k, v.clone())).collect())
            .unwrap_or_default();

        for (target_handle_idx, sources) in connections_to_process {
            let mut source_expressions = Vec::new();
            for (source_node_id, source_handle_idx) in &sources {
                let source_node = self.find_node(source_node_id, node_id)?;
                let expr = if source_node.operation_type == "dynamicNode" {
                    self.build_input_source_expr(source_node, *source_handle_idx)?
                } else {
                    // This mutable call is now safe.
                    self.build_ast(source_node_id, node_id)?
                };
                source_expressions.push(expr);
            }

            if let Some(combined) = source_expressions
                .into_iter()
                .reduce(|acc, expr| Expression::Or(Box::new(acc), Box::new(expr)))
            {
                expressions.insert(target_handle_idx, combined);
            }
        }
        Ok(expressions)
    }

    /// Creates an `Expression::Input` from a "dynamicNode".
    fn build_input_source_expr(
        &self,
        source_node: &FlowNodeDefinition,
        source_handle_idx: u32,
    ) -> Result<Expression, CompileError> {
        let fields =
            source_node
                .data_fields
                .as_ref()
                .ok_or_else(|| CompileError::ConnectionError {
                    target_node_id: source_node.id.clone(),
                    target_handle_index: source_handle_idx,
                    message: "Source data node has no data_fields defined".to_string(),
                })?;

        let field = fields
            .iter()
            .find(|f| f.id == source_handle_idx)
            .ok_or_else(|| CompileError::ConnectionError {
                target_node_id: source_node.id.clone(),
                target_handle_index: source_handle_idx,
                message: format!(
                    "Source handle index {} not found in data_fields",
                    source_handle_idx
                ),
            })?;

        let source = if let Some(event_type) = &source_node.input_type {
            InputSource::Dynamic {
                event: event_type.clone(),
                field: field.name.clone(),
            }
        } else {
            InputSource::Static {
                name: field.name.clone(),
            }
        };
        Ok(Expression::Input(source))
    }

    fn find_node<'b>(
        &self,
        node_id: &'b str,
        source_node_id: &'b str,
    ) -> Result<&'a FlowNodeDefinition, CompileError> {
        self.flow
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| CompileError::NodeNotFound {
                missing_node_id: node_id.to_string(),
                source_node_id: source_node_id.to_string(),
            })
    }

    fn parse_handle_index(handle: &str) -> u32 {
        handle.split('-').last().unwrap_or("0").parse().unwrap_or(0)
    }
}
