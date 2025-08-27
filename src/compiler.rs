use crate::ast::*;
use crate::error::CompileError;
use crate::ui::*;
use std::collections::HashMap;
#[cfg(feature = "hantei-cli")]
use std::fs;

/// Compiles UI recipes into optimized Abstract Syntax Trees
pub struct Compiler {
    recipe: UiRecipe,
    qualities: Vec<Quality>,
    // Maps target node ID -> input index -> Vec<(source node ID, source index)>
    connections: HashMap<String, HashMap<u32, Vec<(String, u32)>>>,
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
        let mut connections: HashMap<String, HashMap<u32, Vec<(String, u32)>>> = HashMap::new();
        for edge in &recipe.edges {
            let target_handle_idx = Self::parse_handle_index(&edge.target_handle);
            let source_handle_idx = Self::parse_handle_index(&edge.source_handle);

            connections
                .entry(edge.target.clone())
                .or_default()
                .entry(target_handle_idx)
                .or_default()
                .push((edge.source.clone(), source_handle_idx));
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

        let quality_node_id = {
            let quality_node = self
                .recipe
                .nodes
                .iter()
                .find(|n| n.data.node_data.real_node_type == "setQualityNode")
                .ok_or_else(|| {
                    CompileError::InvalidNodeType("setQualityNode not found".to_string())
                })?;
            quality_node.id.clone()
        };

        let naive_ast_map = self.gather_inputs_as_map(&quality_node_id)?;

        let mut quality_asts = Vec::new();

        // Iterate through the qualities list to maintain order and context
        for (index, quality) in self.qualities.iter().enumerate() {
            let handle_index = index as u32;

            // Only proceed if the map contains an AST for this quality's index.
            if let Some(naive_ast) = naive_ast_map.get(&handle_index) {
                // If the AST is just a literal null, it's also an effectively empty path. Skip it.
                if let Expression::Literal(Value::Null) = naive_ast {
                    continue;
                }

                let optimized_ast = self.optimize_ast(naive_ast.clone());

                #[cfg(feature = "hantei-cli")]
                {
                    let sanitized_name = Self::sanitize_filename(&quality.name);
                    Self::write_debug_file(
                        &format!("tmp/quality_{}_naive_ast.txt", &sanitized_name),
                        &naive_ast.to_string(),
                    )?;
                    Self::write_debug_file(
                        &format!("tmp/quality_{}_optimized_ast.txt", &sanitized_name),
                        &optimized_ast.to_string(),
                    )?;
                }

                quality_asts.push((quality.priority, quality.name.clone(), optimized_ast));
            }
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

        let expression = match node_data.real_node_type.as_str() {
            "dynamicNode" => {
                if node_data.real_input_type.is_some() {
                    let inputs = self.gather_inputs_as_map(node_id)?;
                    // A pass-through dynamic node should only have one input connection (at index 0)
                    inputs.get(&0).cloned().ok_or_else(|| {
                        CompileError::InvalidInputRef(format!(
                            "Dynamic node '{}' is missing its input connection",
                            node_id
                        ))
                    })?
                } else {
                    return Err(CompileError::InvalidNodeType(format!(
                        "Cannot build a standalone AST for the root input node '{}'",
                        node_id
                    )));
                }
            }
            _ => {
                let inputs = self.gather_inputs_for_node(node_id)?;
                match node_data.real_node_type.as_str() {
                    "gtNode" => Expression::GreaterThan(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "stNode" => Expression::SmallerThan(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "gteqNode" => Expression::GreaterThanOrEqual(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "steqNode" => Expression::SmallerThanOrEqual(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "eqNode" => {
                        Expression::Equal(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
                    }
                    "orNode" => {
                        Expression::Or(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
                    }
                    "andNode" => {
                        Expression::And(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
                    }
                    "sumNode" => {
                        Expression::Sum(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
                    }
                    "subNode" => Expression::Subtract(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "multNode" => Expression::Multiply(
                        Box::new(inputs[0].clone()),
                        Box::new(inputs[1].clone()),
                    ),
                    "divideNode" => {
                        Expression::Divide(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
                    }
                    "notNode" => Expression::Not(Box::new(inputs[0].clone())),
                    _ => return Err(CompileError::InvalidNodeType(node_id.to_string())),
                }
            }
        };

        self.ast_cache
            .insert(node_id.to_string(), expression.clone());
        Ok(expression)
    }

    /// Gathers all inputs for a node into a HashMap, which preserves sparse indices.
    /// This is the primary function for building connections.
    fn gather_inputs_as_map(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<u32, Expression>, CompileError> {
        let node_data = self.find_node(node_id)?.data.node_data.clone();
        let mut expressions: HashMap<u32, Expression> = HashMap::new();

        let connections_to_process: Vec<(u32, Vec<(String, u32)>)> = self
            .connections
            .get(node_id)
            .map(|conn_map| conn_map.iter().map(|(k, v)| (*k, v.clone())).collect())
            .unwrap_or_default();

        for (target_handle_idx, sources) in connections_to_process {
            let mut source_expressions = Vec::new();
            for (source_node_id, source_handle_idx) in sources {
                let source_node_data = self.find_node(&source_node_id)?.data.node_data.clone();
                let expr = if source_node_data.real_node_type == "dynamicNode" {
                    let cases = source_node_data
                        .cases
                        .as_ref()
                        .ok_or_else(|| CompileError::InvalidNodeType(source_node_id.clone()))?;
                    let case = &cases[source_handle_idx as usize];
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
                source_expressions.push(expr);
            }

            if let Some(combined_expr) = source_expressions
                .into_iter()
                .reduce(|acc, expr| Expression::Or(Box::new(acc), Box::new(expr)))
            {
                expressions.insert(target_handle_idx, combined_expr);
            }
        }

        if let Some(values) = node_data.values {
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

    /// Gather all inputs for a node, combining them with OR if necessary
    fn gather_inputs_for_node(&mut self, node_id: &str) -> Result<Vec<Expression>, CompileError> {
        let node_data = self.find_node(node_id)?.data.node_data.clone();
        let mut expressions: HashMap<u32, Expression> = HashMap::new();

        let connections_to_process: Vec<(u32, Vec<(String, u32)>)> = self
            .connections
            .get(node_id)
            .map(|conn_map| conn_map.iter().map(|(k, v)| (*k, v.clone())).collect())
            .unwrap_or_default();

        // This loop no longer holds an immutable borrow on `self`
        for (target_handle_idx, sources) in connections_to_process {
            let mut source_expressions = Vec::new();
            for (source_node_id, source_handle_idx) in sources {
                let source_node_data = self.find_node(&source_node_id)?.data.node_data.clone();
                let expr = if source_node_data.real_node_type == "dynamicNode" {
                    let cases = source_node_data
                        .cases
                        .as_ref()
                        .ok_or_else(|| CompileError::InvalidNodeType(source_node_id.clone()))?;
                    let case = &cases[source_handle_idx as usize];
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
                    // This mutable call is now safe
                    self.build_ast_for_node(&source_node_id)?
                };
                source_expressions.push(expr);
            }

            if let Some(combined_expr) = source_expressions
                .into_iter()
                .reduce(|acc, expr| Expression::Or(Box::new(acc), Box::new(expr)))
            {
                expressions.insert(target_handle_idx, combined_expr);
            }
        }

        if let Some(values) = node_data.values {
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
    /// This helper is only compiled with the 'hantei-cli' feature.
    #[cfg(feature = "hantei-cli")]
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }

    /// Write debug file, creating directory if needed
    /// This helper is only compiled with the 'hantei-cli' feature.
    #[cfg(feature = "hantei-cli")]
    fn write_debug_file(path: &str, content: &str) -> Result<(), CompileError> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CompileError::JsonParseError(format!("Failed to create debug directory: {}", e))
            })?;
        }
        fs::write(path, content)
            .map_err(|e| CompileError::JsonParseError(format!("Failed to write debug file: {}", e)))
    }
}
