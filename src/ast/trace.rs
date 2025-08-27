use super::Value;

/// A record of how an expression was evaluated, including intermediate values.
#[derive(Debug, Clone)]
pub enum EvaluationTrace {
    BinaryOp {
        op_symbol: &'static str,
        left: Box<EvaluationTrace>,
        right: Box<EvaluationTrace>,
        outcome: Value,
    },
    UnaryOp {
        op_symbol: &'static str,
        child: Box<EvaluationTrace>,
        outcome: Value,
    },
    Leaf {
        source: String,
        value: Value,
    },
    NotEvaluated,
}

impl EvaluationTrace {
    pub fn get_outcome(&self) -> Value {
        match self {
            EvaluationTrace::BinaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::UnaryOp { outcome, .. } => outcome.clone(),
            EvaluationTrace::Leaf { value, .. } => value.clone(),
            EvaluationTrace::NotEvaluated => Value::Null,
        }
    }

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
                _ => 0,
            },
            EvaluationTrace::UnaryOp { .. } => 8,
            EvaluationTrace::Leaf { .. } | EvaluationTrace::NotEvaluated => 9,
        }
    }
}
