use crate::ast::*;
use crate::error::CompileError;
use std::collections::HashMap;
use std::fs;

/// The "Compiler" for a recipe.
/// It parses the UI JSON, builds the logical graph,
/// and compiles it into optimized Abstract Syntax Trees (ASTs).
pub struct Compiler {
    recipe: UiRecipe,
    qualities: Vec<Quality>,
    // Key: target_handle_idx, Value: (source_node_id, source_handle_idx)
    connections: HashMap<String, HashMap<u32, (String, u32)>>,
    // Memoization cache for AST building to avoid re-computing branches
    ast_cache: HashMap<String, Expression>,
}

impl Compiler {
    pub fn new(recipe_json: &str, qualities_json: &str) -> Result<Self, CompileError> {
        let recipe: UiRecipe = serde_json::from_str(recipe_json)
            .map_err(|e| CompileError::JsonParseError(e.to_string()))?;
        let qualities: Vec<Quality> = serde_json::from_str(qualities_json)
            .map_err(|e| CompileError::JsonParseError(e.to_string()))?;

        // Pre-process edges into an efficient lookup map
        let mut connections: HashMap<String, HashMap<u32, (String, u32)>> = HashMap::new();
        for edge in &recipe.edges {
            let target_handle_idx = edge
                .target_handle
                .split('-')
                .last()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let source_handle_idx = edge
                .source_handle
                .split('-')
                .last()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

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

    /// The main compilation function.
    pub fn compile(mut self) -> Result<(String, Vec<(i32, String, Expression)>), CompileError> {
        let logical_repr = format!("{:#?}", self.connections);

        let quality_triggers: Vec<_> = self
            .qualities
            .iter()
            .filter_map(|quality| {
                let quality_node_id = "0002".to_string();
                if let Some(inputs_map) = self.connections.get(&quality_node_id) {
                    let quality_input_idx = (quality.priority - 1) as u32;
                    if let Some((source_node_id, _)) = inputs_map.get(&quality_input_idx) {
                        return Some((
                            quality.priority,
                            quality.name.clone(),
                            source_node_id.clone(),
                        ));
                    }
                }
                None
            })
            .collect();

        let mut quality_asts = Vec::new();

        for (priority, name, source_node_id) in quality_triggers {
            log::info!(
                "Compiling AST for Quality '{}' (Priority {})",
                name,
                priority
            );
            let naive_ast = self.build_ast_for_node(&source_node_id)?;
            log::debug!("--- Naive AST for '{}' ---\n{}", name, naive_ast);

            log::debug!("--- Logical Representation ---\n{}", logical_repr);
            let naive_ast_str = naive_ast.to_string();
            // Sanitize filename (e.g., "Qualität A" -> "Qualitat_A")
            let sanitized_name = name
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            let naive_path = format!("tmp/quality_{}_naive_ast.txt", sanitized_name);
            fs::write(&naive_path, &naive_ast_str).expect("Unable to write naive AST to file");
            log::info!("  -> Wrote naive AST to '{}'", naive_path);

            let optimized_ast = self.optimize_ast(naive_ast);
            log::debug!("--- Optimized AST for '{}' ---\n{}", name, optimized_ast);
            let optimized_ast_str = optimized_ast.to_string();
            let opt_path = format!("tmp/quality_{}_optimized_ast.txt", sanitized_name);
            fs::write(&opt_path, &optimized_ast_str)
                .expect("Unable to write optimized AST to file");
            log::info!("  -> Wrote optimized AST to '{}'", opt_path);

            quality_asts.push((priority, name.clone(), optimized_ast));
        }

        quality_asts.sort_by_key(|(p, _, _)| *p);
        Ok((logical_repr, quality_asts))
    }

    /// Finds all inputs for a node, either by recursively building their ASTs
    /// or by parsing them as literal values.
    fn gather_inputs_for_node(&mut self, node_id: &str) -> Result<Vec<Expression>, CompileError> {
        let node_values = self.find_node(node_id)?.data.node_data.values.clone();
        let mut expressions: HashMap<u32, Expression> = HashMap::new();

        // 1. Read all connection data into a new Vec, releasing the immutable borrow on `self`.
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

        // 2. Now, loop over the *new* `connections_data` Vec. `self` is no longer borrowed.
        for (target_handle_idx, source_node_id, source_handle_idx) in connections_data {
            let source_node_data = self.find_node(&source_node_id)?.data.node_data.clone();

            let expr = if source_node_data.real_node_type == "dynamicNode" {
                let cases = source_node_data
                    .cases
                    .as_ref()
                    .ok_or_else(|| CompileError::InvalidNodeType(source_node_id.clone()))?;
                let case = &cases[source_handle_idx as usize];

                // We use the `real_input_type` of the source node to distinguish
                // a static Start node from a dynamic Event node.
                let source = if let Some(event_type) = &source_node_data.real_input_type {
                    // This node is a specific event source, like "hole" or "brown_rot"
                    InputSource::Dynamic {
                        event: event_type.clone(),
                        field: case.case_name.clone(),
                    }
                } else {
                    // The `real_input_type` is null, so this must be the Start node.
                    // Its outputs are static.
                    InputSource::Static {
                        name: case.case_name.clone(),
                    }
                };
                Expression::Input(source)
            } else {
                // 3. This mutable call is now safe because the loop is not borrowing `self`.
                self.build_ast_for_node(&source_node_id)?
            };
            expressions.insert(target_handle_idx, expr);
        }

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

        let mut sorted_expressions: Vec<_> = expressions.into_iter().collect();
        sorted_expressions.sort_by_key(|(idx, _)| *idx);
        Ok(sorted_expressions
            .into_iter()
            .map(|(_, expr)| expr)
            .collect())
    }

    /// Recursively builds an AST for a given node ID.
    fn build_ast_for_node(&mut self, node_id: &str) -> Result<Expression, CompileError> {
        if let Some(cached) = self.ast_cache.get(node_id) {
            return Ok(cached.clone());
        }

        let node_data = self.find_node(node_id)?.data.node_data.clone();
        let node_type = node_data.real_node_type;

        // The mutable part happens here, so all immutable borrows of `self` must be finished.
        let inputs = self.gather_inputs_for_node(node_id)?;

        let expression = match node_type.as_str() {
            "gtNode" => {
                Expression::GreaterThan(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
            "stNode" => {
                Expression::SmallerThan(Box::new(inputs[0].clone()), Box::new(inputs[1].clone()))
            }
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
            "eqNode" => Expression::Equal(Box::new(inputs[0].clone()), Box::new(inputs[1].clone())),
            "steqNode" => Expression::SmallerThanOrEqual(
                Box::new(inputs[0].clone()),
                Box::new(inputs[1].clone()),
            ),
            "gteqNode" => Expression::GreaterThanOrEqual(
                Box::new(inputs[0].clone()),
                Box::new(inputs[1].clone()),
            ),
            "notNode" => Expression::Not(Box::new(inputs[0].clone())),

            "dynamicNode" => {
                // This logic is now handled in `gather_inputs_for_node`, where the connection context is known.
                // Here, we can create a placeholder or error, as a 'dynamicNode' isn't an operation itself.
                // For now, let's create a placeholder representing the first possible output.
                let cases = node_data
                    .cases
                    .as_ref()
                    .ok_or_else(|| CompileError::InvalidNodeType(node_id.to_string()))?;
                let first_case = cases
                    .get(0)
                    .ok_or_else(|| CompileError::InvalidNodeType(node_id.to_string()))?;

                // Use the new optional field
                let event_type = first_case.real_case_type.as_deref().unwrap_or_default();
                let field_name = &first_case.case_name;

                Expression::Input(InputSource::Dynamic {
                    event: event_type.to_string(),
                    field: field_name.clone(),
                })
            }
            _ => return Err(CompileError::InvalidNodeType(node_id.to_string())),
        };

        self.ast_cache
            .insert(node_id.to_string(), expression.clone());
        Ok(expression)
    }

    /// Recursively optimizes an AST.
    fn optimize_ast(&self, expr: Expression) -> Expression {
        // A helper macro to avoid repetition for binary operations
        macro_rules! optimize_binary {
            ($op:ident, $l:expr, $r:expr) => {{
                let opt_l = self.optimize_ast(*$l);
                let opt_r = self.optimize_ast(*$r);
                // Your constant folding logic for this op goes here
                Expression::$op(Box::new(opt_l), Box::new(opt_r))
            }};
        }

        match expr {
            Expression::GreaterThan(l, r) => {
                let opt_l = self.optimize_ast(*l);
                let opt_r = self.optimize_ast(*r);
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
            // Add optimization rules for all other variants
            Expression::Sum(l, r) => optimize_binary!(Sum, l, r),
            Expression::Subtract(l, r) => optimize_binary!(Subtract, l, r),
            Expression::Multiply(l, r) => optimize_binary!(Multiply, l, r),
            Expression::Divide(l, r) => optimize_binary!(Divide, l, r),
            Expression::Abs(v) => Expression::Abs(Box::new(self.optimize_ast(*v))),
            Expression::Not(v) => Expression::Not(Box::new(self.optimize_ast(*v))),
            Expression::And(l, r) => optimize_binary!(And, l, r),
            Expression::Xor(l, r) => optimize_binary!(Xor, l, r),
            Expression::Equal(l, r) => optimize_binary!(Equal, l, r),
            Expression::NotEqual(l, r) => optimize_binary!(NotEqual, l, r),
            Expression::GreaterThanOrEqual(l, r) => optimize_binary!(GreaterThanOrEqual, l, r),
            Expression::SmallerThan(l, r) => optimize_binary!(SmallerThan, l, r),
            Expression::SmallerThanOrEqual(l, r) => optimize_binary!(SmallerThanOrEqual, l, r),
            // Leaf nodes don't need optimization
            _ => expr,
        }
    }

    fn find_node(&self, node_id: &str) -> Result<&UiNode, CompileError> {
        self.recipe
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| CompileError::NodeNotFound(node_id.to_string()))
    }
}
