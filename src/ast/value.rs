use std::fmt;
use std::hash::{Hash, Hasher};

/// Runtime value types used during evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Null,
}

// Manual implementation to handle f64
impl Eq for Value {}

// Manual implementation to handle f64 by hashing its bits
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Number(n) => n.to_bits().hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Null => {} // Null has no data to hash
        }
    }
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
