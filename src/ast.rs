use std::collections::HashSet;
use std::fmt;

/// Runtime value types in the AST evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

/// Defines where leaf nodes get their data from
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

/// Abstract Syntax Tree representing compiled expressions
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

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_pretty(0))
    }
}

impl Expression {
    /// Pretty-print the AST with proper indentation
    fn to_string_pretty(&self, indent: usize) -> String {
        let prefix = "  ".repeat(indent);

        let format_binary = |op: &str, l: &Expression, r: &Expression| {
            format!(
                "{}{} (\n{},\n{}\n{})",
                prefix,
                op,
                l.to_string_pretty(indent + 1),
                r.to_string_pretty(indent + 1),
                prefix
            )
        };

        let format_unary = |op: &str, v: &Expression| {
            format!(
                "{}{}(\n{}\n{})",
                prefix,
                op,
                v.to_string_pretty(indent + 1),
                prefix
            )
        };

        match self {
            // Arithmetic
            Expression::Sum(l, r) => format_binary("+", l, r),
            Expression::Subtract(l, r) => format_binary("-", l, r),
            Expression::Multiply(l, r) => format_binary("*", l, r),
            Expression::Divide(l, r) => format_binary("/", l, r),
            Expression::Abs(v) => format_unary("ABS", v),

            // Logic
            Expression::Not(v) => format_unary("NOT", v),
            Expression::And(l, r) => format_binary("AND", l, r),
            Expression::Or(l, r) => format_binary("OR", l, r),
            Expression::Xor(l, r) => format_binary("XOR", l, r),

            // Comparison
            Expression::Equal(l, r) => format_binary("==", l, r),
            Expression::NotEqual(l, r) => format_binary("!=", l, r),
            Expression::GreaterThan(l, r) => format_binary(">", l, r),
            Expression::GreaterThanOrEqual(l, r) => format_binary(">=", l, r),
            Expression::SmallerThan(l, r) => format_binary("<", l, r),
            Expression::SmallerThanOrEqual(l, r) => format_binary("<=", l, r),

            // Leaves
            Expression::Literal(v) => format!("{}{}\n", prefix, v),
            Expression::Input(s) => format!("{}{}\n", prefix, s),
        }
    }

    /// Find all dynamic event types required by this AST
    pub fn get_required_events(&self, events: &mut HashSet<String>) {
        match self {
            Expression::Input(InputSource::Dynamic { event, .. }) => {
                events.insert(event.clone());
            }
            // Recurse for binary operations
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
            // Recurse for unary operations
            Expression::Abs(v) | Expression::Not(v) => {
                v.get_required_events(events);
            }
            // Leaf nodes don't need recursion
            Expression::Literal(_) | Expression::Input(InputSource::Static { .. }) => {}
        }
    }
}

/// Trace of how an expression was evaluated
#[derive(Debug, Clone)]
pub enum EvaluationTrace {
    /// Binary operation with left/right operands
    BinaryOp {
        op_symbol: &'static str,
        left: Box<EvaluationTrace>,
        right: Box<EvaluationTrace>,
        outcome: Value,
    },
    /// Unary operation with single operand
    UnaryOp {
        op_symbol: &'static str,
        child: Box<EvaluationTrace>,
        outcome: Value,
    },
    /// Leaf value with its source
    Leaf { source: String, value: Value },
    /// Branch not evaluated due to short-circuiting
    NotEvaluated,
}

impl EvaluationTrace {
    /// Get the final evaluated value from this trace
    pub fn get_outcome(&self) -> Value {
        match self {
            EvaluationTrace::BinaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::UnaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::Leaf { value, .. } => value.clone(),
            EvaluationTrace::NotEvaluated => Value::Null,
        }
    }
}
