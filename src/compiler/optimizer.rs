use crate::ast::{Expression, Value};

/// Applies optimization passes to an AST to simplify it.
pub(super) struct AstOptimizer;

impl AstOptimizer {
    pub(super) fn new() -> Self {
        Self
    }

    /// Recursively optimizes an expression tree until no more changes can be made.
    /// This method now runs in a loop to ensure optimizations compound.
    pub(super) fn optimize(&self, expr: Expression) -> Expression {
        let mut current_expr = expr;
        loop {
            let optimized_once = self.optimize_pass(current_expr.clone());
            if optimized_once == current_expr {
                // AST has reached a fixed point, no more optimizations can be applied.
                return optimized_once;
            }
            current_expr = optimized_once;
        }
    }

    /// Performs a single optimization pass over the expression tree.
    fn optimize_pass(&self, expr: Expression) -> Expression {
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

        // Now, apply folding rules to the already-optimized children.
        match optimized_expr {
            // --- Arithmetic Folding ---
            Expression::Sum(l, r) => match (*l, *r) {
                (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) => Expression::Literal(Value::Number(lv + rv)),
                (expr, Expression::Literal(Value::Number(rv))) if rv == 0.0 => expr,
                (Expression::Literal(Value::Number(lv)), expr) if lv == 0.0 => expr,
                (opt_l, opt_r) => Expression::Sum(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::Subtract(l, r) => match (*l, *r) {
                (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) => Expression::Literal(Value::Number(lv - rv)),
                (expr, Expression::Literal(Value::Number(rv))) if rv == 0.0 => expr,
                (opt_l, opt_r) => Expression::Subtract(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::Multiply(l, r) => match (*l, *r) {
                (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) => Expression::Literal(Value::Number(lv * rv)),
                (_, Expression::Literal(Value::Number(rv))) if rv == 0.0 => {
                    Expression::Literal(Value::Number(0.0))
                }
                (Expression::Literal(Value::Number(lv)), _) if lv == 0.0 => {
                    Expression::Literal(Value::Number(0.0))
                }
                (expr, Expression::Literal(Value::Number(rv))) if rv == 1.0 => expr,
                (Expression::Literal(Value::Number(lv)), expr) if lv == 1.0 => expr,
                (opt_l, opt_r) => Expression::Multiply(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::Divide(l, r) => match (*l, *r) {
                (Expression::Literal(Value::Number(_)), Expression::Literal(Value::Number(rv)))
                    if rv == 0.0 =>
                {
                    Expression::Literal(Value::Null)
                } // Avoid division by zero
                (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) => Expression::Literal(Value::Number(lv / rv)),
                (expr, Expression::Literal(Value::Number(rv))) if rv == 1.0 => expr,
                (opt_l, opt_r) => Expression::Divide(Box::new(opt_l), Box::new(opt_r)),
            },

            // --- Logical Folding ---
            Expression::Not(v) => match *v {
                Expression::Literal(Value::Bool(b)) => Expression::Literal(Value::Bool(!b)),
                // Double negation elimination
                Expression::Not(inner_v) => *inner_v,
                opt_v => Expression::Not(Box::new(opt_v)),
            },
            Expression::Or(l, r) => match (*l, *r) {
                (_, Expression::Literal(Value::Bool(true)))
                | (Expression::Literal(Value::Bool(true)), _) => {
                    Expression::Literal(Value::Bool(true))
                }
                (expr, Expression::Literal(Value::Bool(false)))
                | (Expression::Literal(Value::Bool(false)), expr) => expr,
                (opt_l, opt_r) => Expression::Or(Box::new(opt_l), Box::new(opt_r)),
            },
            Expression::And(l, r) => match (*l, *r) {
                (_, Expression::Literal(Value::Bool(false)))
                | (Expression::Literal(Value::Bool(false)), _) => {
                    Expression::Literal(Value::Bool(false))
                }
                (expr, Expression::Literal(Value::Bool(true)))
                | (Expression::Literal(Value::Bool(true)), expr) => expr,
                (opt_l, opt_r) => Expression::And(Box::new(opt_l), Box::new(opt_r)),
            },

            // --- Comparison Folding ---
            Expression::Equal(l, r) => {
                if let (Expression::Literal(lv), Expression::Literal(rv)) = (&*l, &*r) {
                    Expression::Literal(Value::Bool(lv == rv))
                } else {
                    Expression::Equal(l, r)
                }
            }
            Expression::NotEqual(l, r) => {
                if let (Expression::Literal(lv), Expression::Literal(rv)) = (&*l, &*r) {
                    Expression::Literal(Value::Bool(lv != rv))
                } else {
                    Expression::NotEqual(l, r)
                }
            }
            Expression::GreaterThan(l, r) => {
                if let (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) = (&*l, &*r)
                {
                    Expression::Literal(Value::Bool(lv > rv))
                } else {
                    Expression::GreaterThan(l, r)
                }
            }
            Expression::SmallerThan(l, r) => {
                if let (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) = (&*l, &*r)
                {
                    Expression::Literal(Value::Bool(lv < rv))
                } else {
                    Expression::SmallerThan(l, r)
                }
            }
            Expression::GreaterThanOrEqual(l, r) => {
                if let (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) = (&*l, &*r)
                {
                    Expression::Literal(Value::Bool(lv >= rv))
                } else {
                    Expression::GreaterThanOrEqual(l, r)
                }
            }
            Expression::SmallerThanOrEqual(l, r) => {
                if let (
                    Expression::Literal(Value::Number(lv)),
                    Expression::Literal(Value::Number(rv)),
                ) = (&*l, &*r)
                {
                    Expression::Literal(Value::Bool(lv <= rv))
                } else {
                    Expression::SmallerThanOrEqual(l, r)
                }
            }

            // If no rules matched, return the expression as is.
            other => other,
        }
    }
}
