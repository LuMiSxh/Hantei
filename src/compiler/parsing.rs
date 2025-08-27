use crate::ast::Expression;
use crate::error::CompileError;
use crate::recipe::FlowNodeDefinition;
use std::collections::HashMap;

/// Defines the contract for parsing a specific `operation_type` into an `Expression`.
pub trait NodeParser: Send + Sync {
    fn node_type(&self) -> &str;
    fn parse(
        &self,
        node: &FlowNodeDefinition,
        inputs: Vec<Expression>,
    ) -> Result<Expression, CompileError>;
}

/// Helper to check for the correct number of inputs.
fn require_inputs(
    inputs: Vec<Expression>,
    count: usize,
    node_type: &str,
) -> Result<Vec<Expression>, CompileError> {
    if inputs.len() != count {
        Err(CompileError::ConnectionError {
            target_node_id: "unknown".to_string(),
            target_handle_index: 0,
            message: format!(
                "{} requires {} inputs, but received {}",
                node_type,
                count,
                inputs.len()
            ),
        })
    } else {
        Ok(inputs)
    }
}

/// Master macro to define all standard node parsers, their registration, and their creation.
macro_rules! define_node_parsers {
    ( $( ($struct_name:ident, $node_type:expr, Unary, $variant:path) ),* $(,)? ; $( ($bi_struct_name:ident, $bi_node_type:expr, Binary, $bi_variant:path) ),* $(,)? ) => {
        // 1. Define all the parser structs and their implementations
        $(
            struct $struct_name;
            impl NodeParser for $struct_name {
                fn node_type(&self) -> &str { $node_type }
                fn parse(&self, _node: &FlowNodeDefinition, inputs: Vec<Expression>) -> Result<Expression, CompileError> {
                    require_inputs(inputs, 1, $node_type).map(|i| $variant(Box::new(i[0].clone())))
                }
            }
        )*
        $(
            struct $bi_struct_name;
            impl NodeParser for $bi_struct_name {
                fn node_type(&self) -> &str { $bi_node_type }
                fn parse(&self, _node: &FlowNodeDefinition, inputs: Vec<Expression>) -> Result<Expression, CompileError> {
                    require_inputs(inputs, 2, $bi_node_type).map(|i| $bi_variant(Box::new(i[0].clone()), Box::new(i[1].clone())))
                }
            }
        )*

        // 2. Define the function to register all default parsers
        pub(super) fn register_default_parsers(registry: &mut HashMap<String, Box<dyn NodeParser>>) {
            $( registry.insert($node_type.to_string(), Box::new($struct_name)); )*
            $( registry.insert($bi_node_type.to_string(), Box::new($bi_struct_name)); )*
        }

        // 3. Define the function to create a parser by its string name
        pub(super) fn create_parser_by_name(name: &str) -> Option<Box<dyn NodeParser>> {
            match name {
                $( $node_type => Some(Box::new($struct_name)), )*
                $( $bi_node_type => Some(Box::new($bi_struct_name)), )*
                _ => None,
            }
        }
    };
}

// Use the macro to define all standard node parsers
define_node_parsers! {
    // Unary Operators
    (NotNodeParser, "notNode", Unary, Expression::Not),
    (AbsNodeParser, "absNode", Unary, Expression::Abs),

    ; // Separator between unary and binary

    // Binary Operators
    (GtNodeParser, "gtNode", Binary, Expression::GreaterThan),
    (StNodeParser, "stNode", Binary, Expression::SmallerThan),
    (GteqNodeParser, "gteqNode", Binary, Expression::GreaterThanOrEqual),
    (SteqNodeParser, "steqNode", Binary, Expression::SmallerThanOrEqual),
    (EqNodeParser, "eqNode", Binary, Expression::Equal),
    (NeqNodeParser, "neqNode", Binary, Expression::NotEqual),
    (AndNodeParser, "andNode", Binary, Expression::And),
    (OrNodeParser, "orNode", Binary, Expression::Or),
    (XorNodeParser, "xorNode", Binary, Expression::Xor),
    (SumNodeParser, "sumNode", Binary, Expression::Sum),
    (SubNodeParser, "subNode", Binary, Expression::Subtract),
    (MultNodeParser, "multNode", Binary, Expression::Multiply),
    (DivideNodeParser, "divideNode", Binary, Expression::Divide)
}
