use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;

// Represents a runtime value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Null,
}

// For pretty-printing values
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

// Defines exactly where a leaf node in the AST gets its data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSource {
    Static { name: String },
    Dynamic { event: String, field: String },
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputSource::Static { name } => write!(f, "${}", name),
            InputSource::Dynamic { event, field } => write!(f, "${}.{}", event, field),
        }
    }
}

// The Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Arithmetic operations
    Sum(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Abs(Box<Expression>),
    // Logical operations
    Not(Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    // Comparison operations
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterThanOrEqual(Box<Expression>, Box<Expression>),
    SmallerThan(Box<Expression>, Box<Expression>),
    SmallerThanOrEqual(Box<Expression>, Box<Expression>),
    // Leaf nodes
    Literal(Value),
    Input(InputSource),
}

// Custom Display implementation for pretty-printing the AST
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_pretty(0))
    }
}

impl Expression {
    // Helper function for recursive, indented printing of the AST
    fn to_string_pretty(&self, indent: usize) -> String {
        let prefix = "  ".repeat(indent);

        let repr = |op: &str, l: &Expression, r: &Expression| {
            format!(
                "{}{} (\n{},\n{}\n{})",
                prefix,
                op,
                l.to_string_pretty(indent + 1),
                r.to_string_pretty(indent + 1),
                prefix
            )
        };

        match self {
            Expression::Sum(l, r) => repr("+", l, r),
            Expression::Subtract(l, r) => repr("-", l, r),
            Expression::Multiply(l, r) => repr("*", l, r),
            Expression::Divide(l, r) => repr("/", l, r),
            Expression::Abs(v) => format!(
                "{}ABS (\n{},\n{})",
                prefix,
                v.to_string_pretty(indent + 1),
                prefix
            ),
            Expression::Not(v) => format!(
                "{}NOT (\n{},\n{})",
                prefix,
                v.to_string_pretty(indent + 1),
                prefix
            ),
            Expression::Xor(l, r) => repr("XOR", l, r),
            Expression::And(l, r) => repr("AND", l, r),
            Expression::Or(l, r) => repr("OR", l, r),
            Expression::Equal(l, r) => repr("==", l, r),
            Expression::NotEqual(l, r) => repr("!=", l, r),
            Expression::GreaterThanOrEqual(l, r) => repr(">=", l, r),
            Expression::SmallerThanOrEqual(l, r) => repr("<=", l, r),
            Expression::GreaterThan(l, r) => repr(">", l, r),
            Expression::SmallerThan(l, r) => repr("<", l, r),
            Expression::Literal(v) => format!("{}{}\n", prefix, v),
            Expression::Input(s) => format!("{}{}\n", prefix, s),
        }
    }

    // Helper to find all unique event types required by an AST
    pub fn get_required_events(&self, events: &mut HashSet<String>) {
        match self {
            Expression::Input(InputSource::Dynamic { event, .. }) => {
                events.insert(event.clone());
            }
            // --- Recurse for all other nodes that have children ---
            Expression::Sum(l, r)
            | Expression::Subtract(l, r)
            | Expression::Multiply(l, r)
            | Expression::Divide(l, r)
            | Expression::And(l, r)
            | Expression::Or(l, r)
            | Expression::Xor(l, r)
            | Expression::Equal(l, r)
            | Expression::NotEqual(l, r)
            | Expression::GreaterThan(l, r)
            | Expression::GreaterThanOrEqual(l, r)
            | Expression::SmallerThan(l, r)
            | Expression::SmallerThanOrEqual(l, r) => {
                l.get_required_events(events);
                r.get_required_events(events);
            }
            Expression::Abs(v) | Expression::Not(v) => {
                v.get_required_events(events);
            }
            Expression::Literal(_) | Expression::Input(InputSource::Static { .. }) => {
                // Leaf nodes, do nothing
            }
        }
    }
}

// --- Structs for deserializing the raw UI JSON ---
// We only define the fields we actually need. `serde` will ignore the rest.

#[derive(Debug, Deserialize, Clone)]
pub struct UiNodeData {
    #[serde(alias = "realNodeType")]
    pub real_node_type: String,
    #[serde(alias = "realInputType")]
    pub real_input_type: Option<String>,
    pub values: Option<Vec<serde_json::Value>>,
    pub cases: Option<Vec<UiNodeCase>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiNodeCase {
    #[serde(alias = "caseId")]
    pub case_id: u32,
    #[serde(alias = "caseName")]
    pub case_name: String,
    #[serde(default)]
    #[serde(alias = "realCaseType")]
    pub real_case_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UiNode {
    pub id: String,
    pub data: UiNodeWrapper,
}

#[derive(Debug, Deserialize)]
pub struct UiNodeWrapper {
    #[serde(alias = "nodeData")]
    pub node_data: UiNodeData,
}

#[derive(Debug, Deserialize)]
pub struct UiEdge {
    pub source: String,
    #[serde(alias = "sourceHandle")]
    pub source_handle: String,
    pub target: String,
    #[serde(alias = "targetHandle")]
    pub target_handle: String,
}

#[derive(Debug, Deserialize)]
pub struct UiRecipe {
    pub nodes: Vec<UiNode>,
    pub edges: Vec<UiEdge>,
}

#[derive(Debug, Deserialize)]
pub struct Quality {
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub enum EvaluationTrace {
    // Represents a binary operation like ">" or "OR"
    BinaryOp {
        op_symbol: &'static str,
        left: Box<EvaluationTrace>,
        right: Box<EvaluationTrace>,
        outcome: Value,
    },
    // Represents a unary operation like "NOT" or "ABS"
    UnaryOp {
        op_symbol: &'static str,
        child: Box<EvaluationTrace>,
        outcome: Value,
    },
    // The leaf of a logical branch
    Leaf {
        source: String, // e.g., "$Humidity" or "25.0"
        value: Value,
    },
    // Represents a branch that was not evaluated due to short-circuiting
    NotEvaluated,
}

impl EvaluationTrace {
    /// Helper function to get the final value out of a trace.
    pub fn get_outcome(&self) -> Value {
        match self {
            EvaluationTrace::BinaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::UnaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::Leaf { value, .. } => value.clone(),
            EvaluationTrace::NotEvaluated => Value::Null, // Should not happen for a final result
        }
    }
}
