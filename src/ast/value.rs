use std::fmt;

/// Runtime value types used during evaluation.
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

/// Defines the source of data for a leaf node in the AST.
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
