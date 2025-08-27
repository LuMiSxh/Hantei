use crate::ast::{Expression, Value};

/// Applies optimization passes to an AST to simplify it.
pub(super) struct AstOptimizer;

impl AstOptimizer {
    pub(super) fn new() -> Self {
        Self
    }

    /// Recursively optimizes an expression tree via constant folding and logical simplification.
    pub(super) fn optimize(&self, expr: Expression) -> Expression {
        let optimized_expr = match expr {
            Expression::Sum(l, r) => {
                Expression::Sum(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Subtract(l, r) => {
                Expression::Subtract(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Multiply(l, r) => {
                Expression::Multiply(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Divide(l, r) => {
                Expression::Divide(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Abs(v) => Expression::Abs(Box::new(self.optimize(*v))),
            Expression::Not(v) => Expression::Not(Box::new(self.optimize(*v))),
            Expression::And(l, r) => {
                Expression::And(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Or(l, r) => {
                Expression::Or(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Xor(l, r) => {
                Expression::Xor(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::Equal(l, r) => {
                Expression::Equal(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::NotEqual(l, r) => {
                Expression::NotEqual(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::GreaterThan(l, r) => {
                Expression::GreaterThan(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::GreaterThanOrEqual(l, r) => Expression::GreaterThanOrEqual(
                Box::new(self.optimize(*l)),
                Box::new(self.optimize(*r)),
            ),
            Expression::SmallerThan(l, r) => {
                Expression::SmallerThan(Box::new(self.optimize(*l)), Box::new(self.optimize(*r)))
            }
            Expression::SmallerThanOrEqual(l, r) => Expression::SmallerThanOrEqual(
                Box::new(self.optimize(*l)),
                Box::new(self.optimize(*r)),
            ),
            other => other,
        };

        match optimized_expr {
            Expression::Sum(l, r) => match (*l, *r) {
                (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) => Expression::Literal(Value::Number(lv + rv)),
                (expr, Expression::Literal(Value::Number(rv))) if rv == 0.0 => expr,
                (Expression::Literal(Value::Number(lv)), expr) if lv == 0.0 => expr,
                (opt_l, opt_r) => Expression::Sum(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::Not(v) => match *v {
                Expression::Literal(Value::Bool(b)) => Expression::Literal(Value::Bool(!b)),
                Expression::Not(inner_v) => *inner_v,
                opt_v => Expression::Not(Box::new(opt_v)),
            },
            Expression::Or(l, r) => match (*l, *r) {
                (_, Expression::Literal(Value::Bool(true))) => {
                    Expression::Literal(Value::Bool(true))
                }
                (Expression::Literal(Value::Bool(true)), _) => {
                    Expression::Literal(Value::Bool(true))
                }
                (expr, Expression::Literal(Value::Bool(false))) => expr,
                (Expression::Literal(Value::Bool(false)), expr) => expr,
                (opt_l, opt_r) => Expression::Or(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::And(l, r) => match (*l, *r) {
                (_, Expression::Literal(Value::Bool(false))) => {
                    Expression::Literal(Value::Bool(false))
                }
                (Expression::Literal(Value::Bool(false)), _) => {
                    Expression::Literal(Value::Bool(false))
                }
                (expr, Expression::Literal(Value::Bool(true))) => expr,
                (Expression::Literal(Value::Bool(true)), expr) => expr,
                (opt_l, opt_r) => Expression::And(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::Equal(l, r) => {
                if let (Expression::Literal(lv), Expression::Literal(rv)) = (&*l, &*r) {
                    Expression::Literal(Value::Bool(lv == rv))
                } else {
                    Expression::Equal(l, r)
                }
            }
            // ... add more constant folding rules for other operations (Subtract, Multiply, etc.)
            other => other,
        }
    }
}
