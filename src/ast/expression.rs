use super::{InputSource, Value};
use std::collections::HashSet;
use std::fmt;

/// The Abstract Syntax Tree representing a compiled expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Arithmetic
    Sum(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Abs(Box<Expression>),

    // Logical
    Not(Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),

    // Comparison
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

impl Expression {
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

// --- Pretty-printing the AST as a tree structure ---

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_tree(f, "", true)
    }
}

impl Expression {
    fn collect_or_operands<'a>(&'a self, operands: &mut Vec<&'a Expression>) {
        if let Expression::Or(l, r) = self {
            l.collect_or_operands(operands);
            r.collect_or_operands(operands);
        } else {
            operands.push(self);
        }
    }

    fn fmt_as_tree(&self, f: &mut fmt::Formatter<'_>, prefix: &str, is_last: bool) -> fmt::Result {
        let line_prefix = if prefix.is_empty() { "" } else { prefix };
        let node_marker = if is_last { "└── " } else { "├── " };
        write!(f, "{}{}", line_prefix, node_marker)?;

        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        match self {
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
            Expression::Literal(v) => writeln!(f, "Literal: {}", v)?,
            Expression::Input(s) => writeln!(f, "Input: {}", s)?,
            Expression::Not(v) => {
                writeln!(f, "notNode (NOT)")?;
                v.fmt_as_tree(f, &child_prefix, true)?;
            }
            Expression::Abs(v) => {
                writeln!(f, "absNode (ABS)")?;
                v.fmt_as_tree(f, &child_prefix, true)?;
            }
            Expression::GreaterThan(l, r) => {
                self.fmt_binary(f, "gtNode (>)", l, r, &child_prefix)?
            }
            Expression::SmallerThan(l, r) => {
                self.fmt_binary(f, "stNode (<)", l, r, &child_prefix)?
            }
            Expression::GreaterThanOrEqual(l, r) => {
                self.fmt_binary(f, "gteqNode (>=)", l, r, &child_prefix)?
            }
            Expression::SmallerThanOrEqual(l, r) => {
                self.fmt_binary(f, "steqNode (<=)", l, r, &child_prefix)?
            }
            Expression::Equal(l, r) => self.fmt_binary(f, "eqNode (==)", l, r, &child_prefix)?,
            Expression::NotEqual(l, r) => {
                self.fmt_binary(f, "neqNode (!=)", l, r, &child_prefix)?
            }
            Expression::Sum(l, r) => self.fmt_binary(f, "sumNode (+)", l, r, &child_prefix)?,
            Expression::Subtract(l, r) => self.fmt_binary(f, "subNode (-)", l, r, &child_prefix)?,
            Expression::Multiply(l, r) => {
                self.fmt_binary(f, "multNode (*)", l, r, &child_prefix)?
            }
            Expression::Divide(l, r) => {
                self.fmt_binary(f, "divideNode (/)", l, r, &child_prefix)?
            }
            Expression::And(l, r) => self.fmt_binary(f, "andNode (AND)", l, r, &child_prefix)?,
            Expression::Xor(l, r) => self.fmt_binary(f, "xorNode (XOR)", l, r, &child_prefix)?,
        }
        Ok(())
    }

    fn fmt_binary(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        l: &Expression,
        r: &Expression,
        prefix: &str,
    ) -> fmt::Result {
        writeln!(f, "{}", name)?;
        l.fmt_as_tree(f, prefix, false)?;
        r.fmt_as_tree(f, prefix, true)?;
        Ok(())
    }
}
