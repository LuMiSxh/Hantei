use crate::ast::{Expression, Value};
use ahash::AHashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A stateful optimizer that applies advanced simplification passes to an AST.
pub struct AstOptimizer {
    /// A cache mapping an expression's hash to a unique ID for CSE.
    cse_cache: AHashMap<u64, u64>,
    /// A map from a unique ID to the actual expression it represents.
    pub definitions: AHashMap<u64, Expression>,
    next_id: u64,
}

impl AstOptimizer {
    pub fn new() -> Self {
        Self {
            cse_cache: AHashMap::new(),
            definitions: AHashMap::new(),
            next_id: 0,
        }
    }

    /// Runs optimization passes in a loop until the AST reaches a fixed point.
    pub fn optimize(&mut self, expr: Expression) -> Expression {
        let mut current_expr = expr;
        loop {
            let pass1 = self.fold_and_eliminate_dead_code(current_expr.clone());
            let pass2 = self.eliminate_common_subexpressions(pass1);

            if pass2 == current_expr {
                return pass2;
            }
            current_expr = pass2;
        }
    }

    /// Pass 1: Constant folding and Dead Code Elimination (DCE).
    fn fold_and_eliminate_dead_code(&self, expr: Expression) -> Expression {
        // First, recursively optimize the children
        let optimized_expr = match expr {
            Expression::And(l, r) => {
                let left = self.fold_and_eliminate_dead_code(*l);
                let right = self.fold_and_eliminate_dead_code(*r);

                // --- Dead Code Elimination ---
                // Rule: ($x > A) AND ($x < B) where A >= B is impossible.
                if let (Expression::GreaterThan(ll, lr), Expression::SmallerThan(rl, rr)) =
                    (&left, &right)
                {
                    if ll == rl {
                        if let (
                            Expression::Literal(Value::Number(a)),
                            Expression::Literal(Value::Number(b)),
                        ) = (&**lr, &**rr)
                        {
                            if a >= b {
                                return Expression::Literal(Value::Bool(false));
                            }
                        }
                    }
                }

                Expression::And(Box::new(left), Box::new(right))
            }
            Expression::Or(l, r) => Expression::Or(
                Box::new(self.fold_and_eliminate_dead_code(*l)),
                Box::new(self.fold_and_eliminate_dead_code(*r)),
            ),
            Expression::Sum(l, r) => Expression::Sum(
                Box::new(self.fold_and_eliminate_dead_code(*l)),
                Box::new(self.fold_and_eliminate_dead_code(*r)),
            ),
            Expression::Subtract(l, r) => Expression::Subtract(
                Box::new(self.fold_and_eliminate_dead_code(*l)),
                Box::new(self.fold_and_eliminate_dead_code(*r)),
            ),
            Expression::Multiply(l, r) => Expression::Multiply(
                Box::new(self.fold_and_eliminate_dead_code(*l)),
                Box::new(self.fold_and_eliminate_dead_code(*r)),
            ),
            Expression::Divide(l, r) => Expression::Divide(
                Box::new(self.fold_and_eliminate_dead_code(*l)),
                Box::new(self.fold_and_eliminate_dead_code(*r)),
            ),
            Expression::Abs(v) => Expression::Abs(Box::new(self.fold_and_eliminate_dead_code(*v))),
            Expression::Not(v) => Expression::Not(Box::new(self.fold_and_eliminate_dead_code(*v))),
            other => other,
        };

        // Second, apply the existing constant folding rules
        self.apply_folding_rules(optimized_expr)
    }

    /// Pass 2: Common Subexpression Elimination (CSE).
    fn eliminate_common_subexpressions(&mut self, expr: Expression) -> Expression {
        let expr = match expr {
            Expression::Sum(l, r) => Expression::Sum(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Subtract(l, r) => Expression::Subtract(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Multiply(l, r) => Expression::Multiply(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Divide(l, r) => Expression::Divide(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Abs(v) => {
                Expression::Abs(Box::new(self.eliminate_common_subexpressions(*v)))
            }
            Expression::Not(v) => {
                Expression::Not(Box::new(self.eliminate_common_subexpressions(*v)))
            }
            Expression::And(l, r) => Expression::And(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Or(l, r) => Expression::Or(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            _ => expr,
        };

        if !matches!(
            &expr,
            Expression::Literal(_) | Expression::Input(_) | Expression::Reference(_)
        ) {
            let mut hasher = DefaultHasher::new();
            expr.hash(&mut hasher);
            let expr_hash = hasher.finish();

            if let Some(id) = self.cse_cache.get(&expr_hash) {
                return Expression::Reference(*id);
            } else {
                let id = self.next_id;
                self.next_id += 1;
                self.cse_cache.insert(expr_hash, id);
                self.definitions.insert(id, expr.clone());
            }
        }
        expr
    }

    fn apply_folding_rules(&self, expr: Expression) -> Expression {
        match expr {
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
            // ... include all other folding rules from original optimizer ...
            other => other,
        }
    }
}
