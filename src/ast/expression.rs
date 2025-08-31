use super::{InputSource, Value};
#[cfg(feature = "debug-tools")]
pub use display_impl::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expression {
    Sum(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Abs(Box<Expression>),
    Not(Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterThanOrEqual(Box<Expression>, Box<Expression>),
    SmallerThan(Box<Expression>, Box<Expression>),
    SmallerThanOrEqual(Box<Expression>, Box<Expression>),
    Literal(Value),
    Input(InputSource),
    Reference(u64),
}

#[cfg(feature = "debug-tools")]
mod display_impl {
    use super::InputId;
    use ahash::AHashMap;
    use std::fmt;

    /// A wrapper to display an expression with its context.
    pub struct DisplayExpression<'a> {
        pub expr: &'a Expression,
        pub definitions: &'a AHashMap<u64, Expression>,
        pub static_map: &'a AHashMap<InputId, String>,
        pub dynamic_map: &'a AHashMap<InputId, String>,
    }

    impl<'a> fmt::Display for DisplayExpression<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.fmt_as_tree(self.expr, f, "", true)
        }
    }

    impl<'a> DisplayExpression<'a> {
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
                    if let Some(def) = self.definitions.get(id) {
                        self.fmt_as_tree(def, f, &child_prefix, true)?;
                    } else {
                        writeln!(f, "{}└── <Unknown Definition>", child_prefix)?;
                    }
                }
                Expression::Literal(v) => writeln!(f, "Literal: {}", v)?,
                Expression::Input(s) => {
                    let display_str = match s {
                        InputSource::Static { id } => {
                            let name = self.static_map.get(id).map(|s| s.as_str()).unwrap_or("?");
                            format!("${} [S{}]", name, id)
                        }
                        InputSource::Dynamic { id } => {
                            let name = self.dynamic_map.get(id).map(|s| s.as_str()).unwrap_or("?");
                            format!("${} [D{}]", name, id)
                        }
                        // These variants are only used pre-interning, so the maps won't exist.
                        // The naive_ast display will use this path.
                        InputSource::StaticName { name } => format!("${}", name),
                        InputSource::DynamicName { event, field } => {
                            format!("${}.{}", event, field)
                        }
                    };
                    writeln!(f, "Input: {}", display_str)?;
                }
                Expression::Not(v) => {
                    writeln!(f, "notNode (NOT)")?;
                    self.fmt_as_tree(v, f, &child_prefix, true)?;
                }
                Expression::Abs(v) => {
                    writeln!(f, "absNode (ABS)")?;
                    self.fmt_as_tree(v, f, &child_prefix, true)?;
                }
                Expression::Sum(l, r) => self.fmt_binary(f, "sumNode (+)", l, r, &child_prefix)?,
                Expression::Subtract(l, r) => {
                    self.fmt_binary(f, "subNode (-)", l, r, &child_prefix)?
                }
                Expression::Multiply(l, r) => {
                    self.fmt_binary(f, "multNode (*)", l, r, &child_prefix)?
                }
                Expression::Divide(l, r) => {
                    self.fmt_binary(f, "divideNode (/)", l, r, &child_prefix)?
                }
                Expression::And(l, r) => {
                    self.fmt_binary(f, "andNode (AND)", l, r, &child_prefix)?
                }
                Expression::Or(l, r) => self.fmt_binary(f, "orNode (OR)", l, r, &child_prefix)?,
                Expression::Xor(l, r) => {
                    self.fmt_binary(f, "xorNode (XOR)", l, r, &child_prefix)?
                }
                Expression::Equal(l, r) => {
                    self.fmt_binary(f, "eqNode (==)", l, r, &child_prefix)?
                }
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
}
