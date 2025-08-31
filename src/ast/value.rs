use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

pub type InputId = u16;

/// Runtime value types used during evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            Value::Null => {}
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
/// Supports both compilation-time string names and runtime IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    // Runtime variants (used after string interning)
    Static { id: InputId },
    Dynamic { id: InputId },

    // Compilation-time variants (used during initial AST building)
    StaticName { name: String },
    DynamicName { event: String, field: String },
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputSource::Static { id } => write!(f, "$Static[{}]", id),
            InputSource::Dynamic { id } => write!(f, "$Dynamic[{}]", id),
            InputSource::StaticName { name } => write!(f, "${}", name),
            InputSource::DynamicName { event, field } => write!(f, "${}.{}", event, field),
        }
    }
}
