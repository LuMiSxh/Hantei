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
        // Start the recursive formatting with the lowest parent precedence.
        self.fmt_with_precedence(f, 0, 0)
    }
}

impl Expression {
    /// Gets the precedence level for an expression type. Higher numbers bind more tightly.
    fn precedence(&self) -> u8 {
        match self {
            Expression::Or(_, _) => 1,
            Expression::And(_, _) => 2,
            Expression::Xor(_, _) => 3,
            Expression::Equal(_, _) | Expression::NotEqual(_, _) => 4,
            Expression::GreaterThan(_, _)
            | Expression::GreaterThanOrEqual(_, _)
            | Expression::SmallerThan(_, _)
            | Expression::SmallerThanOrEqual(_, _) => 5,
            Expression::Sum(_, _) | Expression::Subtract(_, _) => 6,
            Expression::Multiply(_, _) | Expression::Divide(_, _) => 7,
            Expression::Not(_) | Expression::Abs(_) => 8,
            Expression::Literal(_) | Expression::Input(_) => 9,
        }
    }

    /// Recursively formats the expression, adding parentheses only when necessary.
    fn fmt_with_precedence(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        parent_precedence: u8,
    ) -> fmt::Result {
        let current_precedence = self.precedence();
        let needs_parens = current_precedence < parent_precedence;

        if needs_parens {
            write!(f, "(")?;
        }

        match self {
            // Special handling for OR/AND to add newlines for readability
            Expression::Or(l, r) => {
                l.fmt_with_precedence(f, indent, current_precedence)?;
                write!(f, "\n{}OR\n", "  ".repeat(indent))?;
                r.fmt_with_precedence(f, indent, current_precedence)?;
            }
            Expression::And(l, r) => {
                l.fmt_with_precedence(f, indent + 1, current_precedence)?;
                write!(f, " AND ")?;
                r.fmt_with_precedence(f, indent + 1, current_precedence)?;
            }

            // Generic binary operators
            Expression::GreaterThan(l, r) => {
                self.fmt_binary(f, ">", l, r, indent, current_precedence)?
            }
            Expression::SmallerThan(l, r) => {
                self.fmt_binary(f, "<", l, r, indent, current_precedence)?
            }
            Expression::GreaterThanOrEqual(l, r) => {
                self.fmt_binary(f, ">=", l, r, indent, current_precedence)?
            }
            Expression::SmallerThanOrEqual(l, r) => {
                self.fmt_binary(f, "<=", l, r, indent, current_precedence)?
            }
            Expression::Equal(l, r) => {
                self.fmt_binary(f, "==", l, r, indent, current_precedence)?
            }
            Expression::NotEqual(l, r) => {
                self.fmt_binary(f, "!=", l, r, indent, current_precedence)?
            }
            Expression::Sum(l, r) => self.fmt_binary(f, "+", l, r, indent, current_precedence)?,
            Expression::Subtract(l, r) => {
                self.fmt_binary(f, "-", l, r, indent, current_precedence)?
            }
            Expression::Multiply(l, r) => {
                self.fmt_binary(f, "*", l, r, indent, current_precedence)?
            }
            Expression::Divide(l, r) => {
                self.fmt_binary(f, "/", l, r, indent, current_precedence)?
            }
            Expression::Xor(l, r) => self.fmt_binary(f, "XOR", l, r, indent, current_precedence)?,

            // Unary operators
            Expression::Not(v) => {
                write!(f, "NOT ")?;
                v.fmt_with_precedence(f, indent, current_precedence)?;
            }
            Expression::Abs(v) => {
                write!(f, "ABS")?;
                v.fmt_with_precedence(f, indent, current_precedence)?;
            }

            // Leaf nodes
            Expression::Literal(v) => write!(f, "{}", v)?,
            Expression::Input(s) => write!(f, "{}", s)?,
        }

        if needs_parens {
            write!(f, ")")?;
        }
        Ok(())
    }

    /// Helper function to format a generic binary expression.
    fn fmt_binary(
        &self,
        f: &mut fmt::Formatter<'_>,
        op: &str,
        l: &Expression,
        r: &Expression,
        indent: usize,
        current_precedence: u8,
    ) -> fmt::Result {
        l.fmt_with_precedence(f, indent, current_precedence)?;
        write!(f, " {} ", op)?;
        r.fmt_with_precedence(f, indent, current_precedence)?;
        Ok(())
    }

    // This function is still required by the evaluator and should be kept.
    pub fn get_required_events(&self, events: &mut HashSet<String>) {
        match self {
            Expression::Input(InputSource::Dynamic { event, .. }) => {
                events.insert(event.clone());
            }
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

    /// Gets the precedence level for a trace node. Mirrors `Expression::precedence`.
    pub fn precedence(&self) -> u8 {
        match self {
            EvaluationTrace::BinaryOp { op_symbol, .. } => match *op_symbol {
                "OR" => 1,
                "AND" => 2,
                "XOR" => 3,
                "==" | "!=" => 4,
                ">" | ">=" | "<" | "<=" => 5,
                "+" | "-" => 6,
                "*" | "/" => 7,
                _ => 0, // Default for unknown operators
            },
            EvaluationTrace::UnaryOp { .. } => 8,
            EvaluationTrace::Leaf { .. } | EvaluationTrace::NotEvaluated => 9,
        }
    }
}
