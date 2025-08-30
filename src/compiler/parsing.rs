use crate::ast::Expression;
use crate::error::AstBuildError;
use crate::recipe::FlowNodeDefinition;
use std::collections::HashMap;

/// Defines the contract for parsing a specific `operation_type` into an `Expression`.
pub trait NodeParser: Send + Sync {
    fn node_type(&self) -> &str;
    fn parse(
        &self,
        node: &FlowNodeDefinition,
        inputs: Vec<Expression>,
    ) -> Result<Expression, AstBuildError>;
}

/// Helper to check for at least a certain number of inputs.
fn require_inputs_at_least(
    node: &FlowNodeDefinition,
    inputs: Vec<Expression>,
    count: usize,
) -> Result<Vec<Expression>, AstBuildError> {
    if inputs.len() < count {
        Err(AstBuildError::ConnectionError {
            target_node_id: node.id.clone(),
            target_handle_index: 0,
            message: format!(
                "{} requires at least {} inputs, but received {}",
                node.operation_type,
                count,
                inputs.len()
            ),
        })
    } else {
        Ok(inputs)
    }
}

/// Master macro to define variadic node parsers with different chaining strategies.
macro_rules! define_variadic_parser {
    // ---- Strategy 1: Associative Chaining ----
    // For +, *, AND, OR, -, / where inputs are reduced left-to-right.
    // e.g., [a, b, c] -> Op(Op(a, b), c)
    ( $struct_name:ident, $node_type:expr, $variant:path, Associative ) => {
        struct $struct_name;
        impl NodeParser for $struct_name {
            fn node_type(&self) -> &str {
                $node_type
            }
            fn parse(
                &self,
                node: &FlowNodeDefinition,
                inputs: Vec<Expression>,
            ) -> Result<Expression, AstBuildError> {
                let inputs = require_inputs_at_least(node, inputs, 2)?;
                Ok(inputs
                    .into_iter()
                    .reduce(|acc, expr| $variant(Box::new(acc), Box::new(expr)))
                    .unwrap()) // Safe due to require_inputs_at_least(2)
            }
        }
    };

    // ---- Strategy 2: Chained Comparison ----
    // For >, <, == etc. where inputs are compared pairwise and ANDed.
    // e.g., [a, b, c] -> AND( Op(a,b), Op(b,c) )
    ( $struct_name:ident, $node_type:expr, $variant:path, ChainedComparison ) => {
        struct $struct_name;
        impl NodeParser for $struct_name {
            fn node_type(&self) -> &str {
                $node_type
            }
            fn parse(
                &self,
                node: &FlowNodeDefinition,
                inputs: Vec<Expression>,
            ) -> Result<Expression, AstBuildError> {
                let inputs = require_inputs_at_least(node, inputs, 2)?;
                // Use .windows(2) to get overlapping pairs: [a, b], [b, c], ...
                let comparisons = inputs
                    .windows(2)
                    .map(|pair| $variant(Box::new(pair[0].clone()), Box::new(pair[1].clone())))
                    .collect::<Vec<_>>();

                // Chain the comparisons with AND
                Ok(comparisons
                    .into_iter()
                    .reduce(|acc, expr| Expression::And(Box::new(acc), Box::new(expr)))
                    .unwrap()) // Safe
            }
        }
    };

    // ---- Strategy 3: Unary ----
    // For Not, Abs, etc.
    ( $struct_name:ident, $node_type:expr, $variant:path, Unary ) => {
        struct $struct_name;
        impl NodeParser for $struct_name {
            fn node_type(&self) -> &str {
                $node_type
            }
            fn parse(
                &self,
                node: &FlowNodeDefinition,
                mut inputs: Vec<Expression>,
            ) -> Result<Expression, AstBuildError> {
                if inputs.len() != 1 {
                    return Err(AstBuildError::ConnectionError {
                        target_node_id: node.id.clone(),
                        target_handle_index: 0,
                        message: format!(
                            "{} requires 1 input, but received {}",
                            node.operation_type,
                            inputs.len()
                        ),
                    });
                }
                Ok($variant(Box::new(inputs.pop().unwrap())))
            }
        }
    };
}

// --- Define all parsers using the new, powerful macro ---

// Logical
define_variadic_parser!(AndNodeParser, "andNode", Expression::And, Associative);
define_variadic_parser!(OrNodeParser, "orNode", Expression::Or, Associative);
define_variadic_parser!(XorNodeParser, "xorNode", Expression::Xor, Associative);

// Comparison (note the ChainedComparison strategy)
define_variadic_parser!(
    GtNodeParser,
    "gtNode",
    Expression::GreaterThan,
    ChainedComparison
);
define_variadic_parser!(
    StNodeParser,
    "stNode",
    Expression::SmallerThan,
    ChainedComparison
);
define_variadic_parser!(
    GteqNodeParser,
    "gteqNode",
    Expression::GreaterThanOrEqual,
    ChainedComparison
);
define_variadic_parser!(
    SteqNodeParser,
    "steqNode",
    Expression::SmallerThanOrEqual,
    ChainedComparison
);
define_variadic_parser!(EqNodeParser, "eqNode", Expression::Equal, ChainedComparison);
define_variadic_parser!(
    NeqNodeParser,
    "neqNode",
    Expression::NotEqual,
    ChainedComparison
);

// Arithmetic
define_variadic_parser!(SumNodeParser, "sumNode", Expression::Sum, Associative);
define_variadic_parser!(SubNodeParser, "subNode", Expression::Subtract, Associative);
define_variadic_parser!(
    MultNodeParser,
    "multNode",
    Expression::Multiply,
    Associative
);
define_variadic_parser!(
    DivideNodeParser,
    "divideNode",
    Expression::Divide,
    Associative
);

// Unary
define_variadic_parser!(NotNodeParser, "notNode", Expression::Not, Unary);
define_variadic_parser!(AbsNodeParser, "absNode", Expression::Abs, Unary);

/// Adds all defined node parsers to the registry HashMap.
pub(super) fn register_default_parsers(registry: &mut HashMap<String, Box<dyn NodeParser>>) {
    registry.insert("andNode".to_string(), Box::new(AndNodeParser));
    registry.insert("orNode".to_string(), Box::new(OrNodeParser));
    registry.insert("xorNode".to_string(), Box::new(XorNodeParser));
    registry.insert("gtNode".to_string(), Box::new(GtNodeParser));
    registry.insert("stNode".to_string(), Box::new(StNodeParser));
    registry.insert("gteqNode".to_string(), Box::new(GteqNodeParser));
    registry.insert("steqNode".to_string(), Box::new(SteqNodeParser));
    registry.insert("eqNode".to_string(), Box::new(EqNodeParser));
    registry.insert("neqNode".to_string(), Box::new(NeqNodeParser));
    registry.insert("sumNode".to_string(), Box::new(SumNodeParser));
    registry.insert("subNode".to_string(), Box::new(SubNodeParser));
    registry.insert("multNode".to_string(), Box::new(MultNodeParser));
    registry.insert("divideNode".to_string(), Box::new(DivideNodeParser));
    registry.insert("notNode".to_string(), Box::new(NotNodeParser));
    registry.insert("absNode".to_string(), Box::new(AbsNodeParser));
}

/// Creates a parser instance by its string name, used for type mapping.
pub(super) fn create_parser_by_name(name: &str) -> Option<Box<dyn NodeParser>> {
    match name {
        "andNode" => Some(Box::new(AndNodeParser)),
        "orNode" => Some(Box::new(OrNodeParser)),
        "xorNode" => Some(Box::new(XorNodeParser)),
        "gtNode" => Some(Box::new(GtNodeParser)),
        "stNode" => Some(Box::new(StNodeParser)),
        "gteqNode" => Some(Box::new(GteqNodeParser)),
        "steqNode" => Some(Box::new(SteqNodeParser)),
        "eqNode" => Some(Box::new(EqNodeParser)),
        "neqNode" => Some(Box::new(NeqNodeParser)),
        "sumNode" => Some(Box::new(SumNodeParser)),
        "subNode" => Some(Box::new(SubNodeParser)),
        "multNode" => Some(Box::new(MultNodeParser)),
        "divideNode" => Some(Box::new(DivideNodeParser)),
        "notNode" => Some(Box::new(NotNodeParser)),
        "absNode" => Some(Box::new(AbsNodeParser)),
        _ => None,
    }
}
