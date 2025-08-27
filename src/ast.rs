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
        // Start the recursive tree formatting with an empty prefix
        self.fmt_as_tree(f, "", true)
    }
}

impl Expression {
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

    /// Recursively collects all non-OR operands from a nested chain of OR expressions.
    fn collect_or_operands<'a>(&'a self, operands: &mut Vec<&'a Expression>) {
        if let Expression::Or(l, r) = self {
            l.collect_or_operands(operands);
            r.collect_or_operands(operands);
        } else {
            operands.push(self);
        }
    }

    /// Formats the expression as a hierarchical, human-readable tree.
    fn fmt_as_tree(&self, f: &mut fmt::Formatter<'_>, prefix: &str, is_last: bool) -> fmt::Result {
        let line_prefix = if prefix.is_empty() { "" } else { prefix };
        let node_marker = if is_last { "└── " } else { "├── " };
        write!(f, "{}{}", line_prefix, node_marker)?;

        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        match self {
            // SPECIAL CASE: Visually flatten OR chains
            Expression::Or(_, _) => {
                writeln!(f, "orNode (OR)")?;
                let mut operands = Vec::new();
                self.collect_or_operands(&mut operands);
                let num_operands = operands.len();
                for (i, operand) in operands.iter().enumerate() {
                    let is_last_operand = i == num_operands - 1;
                    operand.fmt_as_tree(f, &child_prefix, is_last_operand)?;
                }
            }

            // Other nodes are handled normally
            Expression::Literal(v) => match v {
                Value::Number(_) => writeln!(f, "Literal(Number): {}", v)?,
                Value::Bool(_) => writeln!(f, "Literal(Bool): {}", v)?,
                Value::Null => writeln!(f, "Literal(Null)")?,
            },
            Expression::Input(s) => match s {
                InputSource::Static { .. } => writeln!(f, "Input(Static): {}", s)?,
                InputSource::Dynamic { .. } => writeln!(f, "Input(Dynamic): {}", s)?,
            },
            Expression::Not(v) => {
                writeln!(f, "notNode (NOT)")?;
                v.fmt_as_tree(f, &child_prefix, true)?;
            }
            Expression::Abs(v) => {
                writeln!(f, "absNode (ABS)")?;
                v.fmt_as_tree(f, &child_prefix, true)?;
            }
            Expression::GreaterThan(l, r) => {
                self.fmt_binary_as_tree(f, "gtNode (>)", l, r, &child_prefix)?
            }
            Expression::SmallerThan(l, r) => {
                self.fmt_binary_as_tree(f, "stNode (<)", l, r, &child_prefix)?
            }
            Expression::GreaterThanOrEqual(l, r) => {
                self.fmt_binary_as_tree(f, "gteqNode (>=)", l, r, &child_prefix)?
            }
            Expression::SmallerThanOrEqual(l, r) => {
                self.fmt_binary_as_tree(f, "steqNode (<=)", l, r, &child_prefix)?
            }
            Expression::Equal(l, r) => {
                self.fmt_binary_as_tree(f, "eqNode (==)", l, r, &child_prefix)?
            }
            Expression::NotEqual(l, r) => {
                self.fmt_binary_as_tree(f, "neqNode (!=)", l, r, &child_prefix)?
            }
            Expression::Sum(l, r) => {
                self.fmt_binary_as_tree(f, "sumNode (+)", l, r, &child_prefix)?
            }
            Expression::Subtract(l, r) => {
                self.fmt_binary_as_tree(f, "subNode (-)", l, r, &child_prefix)?
            }
            Expression::Multiply(l, r) => {
                self.fmt_binary_as_tree(f, "multNode (*)", l, r, &child_prefix)?
            }
            Expression::Divide(l, r) => {
                self.fmt_binary_as_tree(f, "divideNode (/)", l, r, &child_prefix)?
            }
            Expression::And(l, r) => {
                self.fmt_binary_as_tree(f, "andNode (AND)", l, r, &child_prefix)?
            }
            Expression::Xor(l, r) => {
                self.fmt_binary_as_tree(f, "xorNode (XOR)", l, r, &child_prefix)?
            }
        }
        Ok(())
    }

    /// Helper for formatting all binary operators except for OR.
    fn fmt_binary_as_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        node_name: &str,
        l: &Expression,
        r: &Expression,
        child_prefix: &str,
    ) -> fmt::Result {
        writeln!(f, "{}", node_name)?;
        l.fmt_as_tree(f, child_prefix, false)?;
        r.fmt_as_tree(f, child_prefix, true)?;
        Ok(())
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
