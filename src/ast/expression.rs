use super::{InputSource, Value};
use ahash::AHashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

/// The Abstract Syntax Tree representing a compiled expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Reference(u64),
}

impl Expression {
    /// Gets required dynamic events from a simple AST tree (after linking).
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
            // After linking, Reference nodes do not exist in the AST passed to the evaluator.
            // But they do exist when we print the optimized AST, so we handle it here.
            Expression::Literal(_)
            | Expression::Input(InputSource::Static { .. })
            | Expression::Reference(_) => {}
        }
    }
}

/// A wrapper to display an expression with its definitions for references.
/// This is crucial for debugging the output of the optimizer.
pub struct DisplayExpression<'a> {
    pub expr: &'a Expression,
    pub definitions: &'a AHashMap<u64, Expression>,
}

impl<'a> fmt::Display for DisplayExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_tree(self.expr, f, "", true)
    }
}

impl<'a> DisplayExpression<'a> {
    /// Recursively formats the AST, resolving and printing references from the definitions map.
    fn fmt_as_tree(
        &self,
        expr: &Expression,
        f: &mut fmt::Formatter<'_>,
        prefix: &str,
        is_last: bool,
    ) -> fmt::Result {
        let line_prefix = if prefix.is_empty() { "" } else { prefix };
        let node_marker = if is_last { "└── " } else { "├── " };
        write!(f, "{}{}", line_prefix, node_marker)?;

        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        match expr {
            Expression::Reference(id) => {
                writeln!(f, "Reference: #{}", id)?;
                // Look up the definition and print its tree structure underneath.
                if let Some(def) = self.definitions.get(id) {
                    self.fmt_as_tree(def, f, &child_prefix, true)?;
                } else {
                    writeln!(f, "{}└── <Unknown Definition>", child_prefix)?;
                }
            }
            Expression::Literal(v) => writeln!(f, "Literal: {}", v)?,
            Expression::Input(s) => writeln!(f, "Input: {}", s)?,
            Expression::Not(v) => {
                writeln!(f, "notNode (NOT)")?;
                self.fmt_as_tree(v, f, &child_prefix, true)?;
            }
            Expression::Abs(v) => {
                writeln!(f, "absNode (ABS)")?;
                self.fmt_as_tree(v, f, &child_prefix, true)?;
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
            Expression::Or(l, r) => self.fmt_binary(f, "orNode (OR)", l, r, &child_prefix)?,
            Expression::Xor(l, r) => self.fmt_binary(f, "xorNode (XOR)", l, r, &child_prefix)?,
            Expression::Equal(l, r) => self.fmt_binary(f, "eqNode (==)", l, r, &child_prefix)?,
            Expression::NotEqual(l, r) => {
                self.fmt_binary(f, "neqNode (!=)", l, r, &child_prefix)?
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
        self.fmt_as_tree(l, f, prefix, false)?;
        self.fmt_as_tree(r, f, prefix, true)?;
        Ok(())
    }
}
