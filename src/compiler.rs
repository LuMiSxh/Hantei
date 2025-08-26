use crate::ast::*;
use crate::error::CompileError;
use crate::ui::*;
use std::collections::HashMap;
use std::fs;

/// Compiles UI recipes into optimized Abstract Syntax Trees
pub struct Compiler {
    recipe: UiRecipe,
    qualities: Vec<Quality>,
    // Maps target node ID -> input index -> (source node ID, source index)
    connections: HashMap<String, HashMap<u32, (String, u32)>>,
    // Cache to avoid recomputing ASTs for the same nodes
    ast_cache: HashMap<String, Expression>,
}

impl Compiler {
    /// Create a new compiler from JSON strings
    pub fn new(recipe_json: &str, qualities_json: &str) -> Result<Self, CompileError> {
        let recipe: UiRecipe = serde_json::from_str(recipe_json)
            .map_err(|e| CompileError::JsonParseError(e.to_string()))?;
        let qualities: Vec<Quality> = serde_json::from_str(qualities_json)
            .map_err(|e| CompileError::JsonParseError(e.to_string()))?;

        // Build connection lookup map for efficient AST traversal
        let mut connections: HashMap<String, HashMap<u32, (String, u32)>> = HashMap::new();
        for edge in &recipe.edges {
            let target_handle_idx = Self::parse_handle_index(&edge.target_handle);
            let source_handle_idx = Self::parse_handle_index(&edge.source_handle);

            connections
                .entry(edge.target.clone())
                .or_default()
                .insert(target_handle_idx, (edge.source.clone(), source_handle_idx));
        }

        Ok(Self {
            recipe,
            qualities,
            connections,
            ast_cache: HashMap::new(),
        })
    }

    /// Compile the recipe into ASTs for each quality
    pub fn compile(mut self) -> Result<(String, Vec<(i32, String, Expression)>), CompileError> {
        let logical_repr = format!("{:#?}", self.connections);

        // Find quality trigger nodes by looking for setQualityNode types
        let mut quality_triggers = Vec::new();

        // Find all setQualityNode nodes
        for node in &self.recipe.nodes {
            let node_data = &node.data.node_data;
            {
                if node_data.real_node_type == "setQualityNode" {
                    // For each quality, check if there's a connection to this setQualityNode
                    for quality in &self.qualities {
                        if let Some(inputs_map) = self.connections.get(&node.id) {
                            // Try to find input connection for this quality
                            // For simplicity, we'll use the first available input connection
                            if let Some((source_node_id, _)) = inputs_map.values().next() {
                                quality_triggers.push((
                                    quality.priority,
                                    quality.name.clone(),
                                    source_node_id.clone(),
                                ));
                                break; // Only one quality per setQualityNode for now
                            }
                        }
                    }
                }
            }
        }

        let mut quality_asts = Vec::new();

        for (priority, name, source_node_id) in quality_triggers {
            // Build naive AST
            let naive_ast = self.build_ast_for_node(&source_node_id)?;

            // Write debug files
            let sanitized_name = Self::sanitize_filename(&name);
            Self::write_debug_file(
                &format!("tmp/quality_{}_naive_ast.txt", sanitized_name),
                &naive_ast.to_string(),
            )?;

            // Optimize AST
            let optimized_ast = self.optimize_ast(naive_ast);
            Self::write_debug_file(
                &format!("tmp/quality_{}_optimized_ast.txt", sanitized_name),
                &optimized_ast.to_string(),
            )?;

            quality_asts.push((priority, name.clone(), optimized_ast));
        }

        quality_asts.sort_by_key(|(p, _, _)| *p);
        Ok((logical_repr, quality_asts))
    }

    /// Build AST for a specific node recursively
    fn build_ast_for_node(&mut self, node_id: &str) -> Result<Expression, CompileError> {
        if let Some(cached) = self.ast_cache.get(node_id) {
            return Ok(cached.clone());
        }

        let node_data = self.find_node(node_id)?.data.node_data.clone();
        let inputs = self.gather_inputs_for_node(node_id)?;

        let expression = match node_data.real_node_type.as_str() {
            "gtNode" => {
                Expression::GreaterThan(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "stNode" => {
                Expression::SmallerThan(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "gteqNode" => Expression::GreaterThanOrEqual(
                Box::new(inputs[0].clone()),
                Box::new(inputs[1].clone()),
            ),
            "steqNode" => Expression::SmallerThanOrEqual(
                Box::new(inputs[0].clone()),
                Box::new(inputs[1].clone()),
            ),
            "eqNode" => Expression::Equal(Box::new(inputs[0].clone()), Box::new(inputs[1].clone())),
            "orNode" => Expression::Or(Box::new(inputs[0].clone()), Box::new(inputs[1].clone())),
            "andNode" => Expression::And(Box::new(inputs[0].clone()), Box::new(inputs[1].clone())),
            "sumNode" => Expression::Sum(Box::new(inputs[0].clone()), Box::new(inputs[1].clone())),
            "subNode" => {
                Expression::Subtract(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "multNode" => {
                Expression::Multiply(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "divideNode" => {
                Expression::Divide(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "notNode" => Expression::Not(Box::new(inputs[0].clone())),
            "dynamicNode" => {
                // Handle dynamic nodes as input sources
                let cases = node_data
                    .cases
                    .as_ref()
                    .ok_or_else(|| CompileError::InvalidNodeType(node_id.to_string()))?;
                let first_case = cases
                    .get(0)
                    .ok_or_else(|| CompileError::InvalidNodeType(node_id.to_string()))?;

                // Determine if this is static or dynamic based on real_input_type
                let source = if let Some(event_type) = &node_data.real_input_type {
                    InputSource::Dynamic {
                        event: event_type.clone(),
                        field: first_case.case_name.clone(),
                    }
                } else {
                    InputSource::Static {
                        name: first_case.case_name.clone(),
                    }
                };
                Expression::Input(source)
            }
            _ => return Err(CompileError::InvalidNodeType(node_id.to_string())),
        };

        self.ast_cache
            .insert(node_id.to_string(), expression.clone());
        Ok(expression)
    }

    /// Gather all inputs for a node (from connections and literal values)
    fn gather_inputs_for_node(&mut self, node_id: &str) -> Result<Vec<Expression>, CompileError> {
        let node_values = self.find_node(node_id)?.data.node_data.values.clone();
        let mut expressions: HashMap<u32, Expression> = HashMap::new();

        // Process connections first
        let connections_data: Vec<(u32, String, u32)> =
            if let Some(connections_map) = self.connections.get(node_id) {
                connections_map
                    .iter()
                    .map(|(target_idx, (source_id, source_idx))| {
                        (*target_idx, source_id.clone(), *source_idx)
                    })
                    .collect()
            } else {
                Vec::new()
            };

        for (target_handle_idx, source_node_id, source_handle_idx) in connections_data {
            let source_node_data = self.find_node(&source_node_id)?.data.node_data.clone();

            let expr = if source_node_data.real_node_type == "dynamicNode" {
                let cases = source_node_data
                    .cases
                    .as_ref()
                    .ok_or_else(|| CompileError::InvalidNodeType(source_node_id.clone()))?;
                let case = &cases[source_handle_idx as usize];

                // Determine if this is static or dynamic based on real_input_type
                let source = if let Some(event_type) = &source_node_data.real_input_type {
                    InputSource::Dynamic {
                        event: event_type.clone(),
                        field: case.case_name.clone(),
                    }
                } else {
                    InputSource::Static {
                        name: case.case_name.clone(),
                    }
                };
                Expression::Input(source)
            } else {
                self.build_ast_for_node(&source_node_id)?
            };
            expressions.insert(target_handle_idx, expr);
        }

        // Fill in literal values from node data
        if let Some(values) = node_values {
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

        // Sort by input index and return as vector
        let mut sorted_expressions: Vec<_> = expressions.into_iter().collect();
        sorted_expressions.sort_by_key(|(idx, _)| *idx);
        Ok(sorted_expressions
            .into_iter()
            .map(|(_, expr)| expr)
            .collect())
    }

    /// Apply optimizations to the AST
    fn optimize_ast(&self, expr: Expression) -> Expression {
        match expr {
            Expression::GreaterThan(l, r) => {
                let opt_l = self.optimize_ast(*l);
                let opt_r = self.optimize_ast(*r);
                // Constant folding for numeric comparisons
                if let (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) = (&opt_l, &opt_r)
                {
                    return Expression::Literal(Value::Bool(lv > rv));
                }
                Expression::GreaterThan(Box::new(opt_l), Box::new(opt_r))
            }
            Expression::Or(l, r) => {
                let opt_l = self.optimize_ast(*l);
                let opt_r = self.optimize_ast(*r);
                // Short-circuit optimizations
                match (&opt_l, &opt_r) {
                    (Expression::Literal(Value::Bool(true)), _) => {
                        Expression::Literal(Value::Bool(true))
                    }
                    (_, Expression::Literal(Value::Bool(true))) => {
                        Expression::Literal(Value::Bool(true))
                    }
                    (Expression::Literal(Value::Bool(false)), _) => opt_r,
                    (_, Expression::Literal(Value::Bool(false))) => opt_l,
                    _ => Expression::Or(Box::new(opt_l), Box::new(opt_r)),
                }
            }
            // Recursively optimize other expressions
            Expression::Sum(l, r) => Expression::Sum(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Subtract(l, r) => Expression::Subtract(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Multiply(l, r) => Expression::Multiply(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Divide(l, r) => Expression::Divide(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Abs(v) => Expression::Abs(Box::new(self.optimize_ast(*v))),
            Expression::Not(v) => Expression::Not(Box::new(self.optimize_ast(*v))),
            Expression::And(l, r) => Expression::And(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Xor(l, r) => Expression::Xor(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::Equal(l, r) => Expression::Equal(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::NotEqual(l, r) => Expression::NotEqual(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::GreaterThanOrEqual(l, r) => Expression::GreaterThanOrEqual(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::SmallerThan(l, r) => Expression::SmallerThan(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            Expression::SmallerThanOrEqual(l, r) => Expression::SmallerThanOrEqual(
                Box::new(self.optimize_ast(*l)),
                Box::new(self.optimize_ast(*r)),
            ),
            // Leaf nodes don't need optimization
            other => other,
        }
    }

    /// Find a node by ID
    fn find_node(&self, node_id: &str) -> Result<&UiNode, CompileError> {
        self.recipe
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| CompileError::NodeNotFound(node_id.to_string()))
    }

    /// Parse handle index from handle string (e.g., "input-0" -> 0)
    fn parse_handle_index(handle: &str) -> u32 {
        handle.split('-').last().unwrap_or("0").parse().unwrap_or(0)
    }

    /// Sanitize filename by removing special characters
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }

    /// Write debug file, creating directory if needed
    fn write_debug_file(path: &str, content: &str) -> Result<(), CompileError> {
        fs::write(path, content)
            .map_err(|e| CompileError::JsonParseError(format!("Failed to write debug file: {}", e)))
    }
}
