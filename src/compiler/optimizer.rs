use crate::ast::{Expression, Value};
use ahash::AHashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A macro to handle simplification rules for any binary expression.
/// It tries to apply a series of patterns and if none match, it reconstructs
/// the expression with its (already optimized) children.
macro_rules! apply_binary_rules {
    // The macro takes the left/right children, the default constructor,
    // and a series of pattern arms. Each arm can now optionally have an `if` guard.
    ($l:expr, $r:expr, $default_constructor:path, $($pattern:pat $(if $guard:expr)? => $result:expr),+ $(,)?) => {
        match (*$l, *$r) {
            $(
                $pattern $(if $guard)? => $result,
            )+
            // Default Case: If no specific rule matches, reconstruct the expression.
            (opt_l, opt_r) => $default_constructor(Box::new(opt_l), Box::new(opt_r)),
        }
    };
}

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
            // It's crucial to run folding/elimination before CSE to maximize cache hits.
            let pass1 = self.fold_and_simplify(current_expr.clone());
            let pass2 = self.eliminate_common_subexpressions(pass1);

            if pass2 == current_expr {
                return pass2;
            }
            current_expr = pass2;
        }
    }

    /// Pass 1: A combined pass for Constant Folding, Algebraic Simplification,
    /// Dead Code Elimination (DCE), and De Morgan's Laws.
    fn fold_and_simplify(&self, expr: Expression) -> Expression {
        // First, recursively optimize the children (post-order traversal).
        let expr = match expr {
            Expression::Sum(l, r) => Expression::Sum(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Subtract(l, r) => Expression::Subtract(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Multiply(l, r) => Expression::Multiply(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Divide(l, r) => Expression::Divide(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::And(l, r) => Expression::And(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Or(l, r) => Expression::Or(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Xor(l, r) => Expression::Xor(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Equal(l, r) => Expression::Equal(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::NotEqual(l, r) => Expression::NotEqual(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::GreaterThan(l, r) => Expression::GreaterThan(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::GreaterThanOrEqual(l, r) => Expression::GreaterThanOrEqual(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::SmallerThan(l, r) => Expression::SmallerThan(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::SmallerThanOrEqual(l, r) => Expression::SmallerThanOrEqual(
                Box::new(self.fold_and_simplify(*l)),
                Box::new(self.fold_and_simplify(*r)),
            ),
            Expression::Not(v) => Expression::Not(Box::new(self.fold_and_simplify(*v))),
            Expression::Abs(v) => Expression::Abs(Box::new(self.fold_and_simplify(*v))),
            other => other,
        };

        // Second, apply simplification rules to the current node.
        self.apply_simplification_rules(expr)
    }

    /// Pass 2: Common Subexpression Elimination (CSE).
    fn eliminate_common_subexpressions(&mut self, expr: Expression) -> Expression {
        // Recursively apply to children first (post-order traversal).
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
            Expression::Xor(l, r) => Expression::Xor(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::Equal(l, r) => Expression::Equal(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::NotEqual(l, r) => Expression::NotEqual(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::GreaterThan(l, r) => Expression::GreaterThan(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::GreaterThanOrEqual(l, r) => Expression::GreaterThanOrEqual(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::SmallerThan(l, r) => Expression::SmallerThan(
                Box::new(self.eliminate_common_subexpressions(*l)),
                Box::new(self.eliminate_common_subexpressions(*r)),
            ),
            Expression::SmallerThanOrEqual(l, r) => Expression::SmallerThanOrEqual(
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

    fn apply_simplification_rules(&self, expr: Expression) -> Expression {
        match expr {
            // --- Arithmetic ---
            Expression::Sum(l, r) => apply_binary_rules!(l, r, Expression::Sum,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Number(lv + rv)),
                (expr, Expression::Literal(Value::Number(n))) if n == 0.0 => expr,
                (Expression::Literal(Value::Number(n)), expr) if n == 0.0 => expr,
            ),
            Expression::Subtract(l, r) => apply_binary_rules!(l, r, Expression::Subtract,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Number(lv - rv)),
                (expr, Expression::Literal(Value::Number(n))) if n == 0.0 => expr,
                (l_expr, r_expr) if l_expr == r_expr => Expression::Literal(Value::Number(0.0)),
            ),
            Expression::Multiply(l, r) => apply_binary_rules!(l, r, Expression::Multiply,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Number(lv * rv)),
                (_, Expression::Literal(Value::Number(n))) if n == 0.0 => Expression::Literal(Value::Number(0.0)),
                (Expression::Literal(Value::Number(n)), _) if n == 0.0 => Expression::Literal(Value::Number(0.0)),
                (expr, Expression::Literal(Value::Number(n))) if n == 1.0 => expr,
                (Expression::Literal(Value::Number(n)), expr) if n == 1.0 => expr,
            ),
            Expression::Divide(l, r) => apply_binary_rules!(l, r, Expression::Divide,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) if rv != 0.0 => Expression::Literal(Value::Number(lv / rv)),
                (expr, Expression::Literal(Value::Number(n))) if n == 1.0 => expr,
                (Expression::Literal(Value::Number(n)), _) if n == 0.0 => Expression::Literal(Value::Number(0.0)),
            ),

            // --- Unary ---
            Expression::Abs(v) => match *v {
                Expression::Literal(Value::Number(n)) => {
                    Expression::Literal(Value::Number(n.abs()))
                }
                opt_v => Expression::Abs(Box::new(opt_v)),
            },
            Expression::Not(v) => match *v {
                Expression::Literal(Value::Bool(b)) => Expression::Literal(Value::Bool(!b)),
                Expression::Not(inner_v) => *inner_v,
                Expression::Or(l, r) => self.fold_and_simplify(Expression::And(
                    Box::new(Expression::Not(l)),
                    Box::new(Expression::Not(r)),
                )),
                Expression::And(l, r) => self.fold_and_simplify(Expression::Or(
                    Box::new(Expression::Not(l)),
                    Box::new(Expression::Not(r)),
                )),
                opt_v => Expression::Not(Box::new(opt_v)),
            },

            // --- Logical ---
            Expression::Or(l, r) => apply_binary_rules!(l, r, Expression::Or,
                (_, Expression::Literal(Value::Bool(true))) | (Expression::Literal(Value::Bool(true)), _) => Expression::Literal(Value::Bool(true)),
                (expr, Expression::Literal(Value::Bool(false))) | (Expression::Literal(Value::Bool(false)), expr) => expr,
                (l_expr, r_expr) if l_expr == r_expr => l_expr,
            ),
            Expression::Xor(l, r) => apply_binary_rules!(l, r, Expression::Xor,
                (Expression::Literal(Value::Bool(lv)), Expression::Literal(Value::Bool(rv))) => Expression::Literal(Value::Bool(lv ^ rv)),
                (expr, Expression::Literal(Value::Bool(false))) | (Expression::Literal(Value::Bool(false)), expr) => expr,
                (expr, Expression::Literal(Value::Bool(true))) => Expression::Not(Box::new(expr)),
                (Expression::Literal(Value::Bool(true)), expr) => Expression::Not(Box::new(expr)),
                (l_expr, r_expr) if l_expr == r_expr => Expression::Literal(Value::Bool(false)),
            ),
            Expression::And(l, r) => {
                // `And` has complex DCE rules that don't fit the simple macro, so it gets a custom match.
                // The simple folding/identity rules are in the default arm.
                match (&*l, &*r) {
                    (Expression::GreaterThan(ll, lr), Expression::SmallerThan(rl, rr))
                        if ll == rl =>
                    {
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
                    (Expression::SmallerThan(ll, lr), Expression::GreaterThan(rl, rr))
                        if ll == rl =>
                    {
                        if let (
                            Expression::Literal(Value::Number(a)),
                            Expression::Literal(Value::Number(b)),
                        ) = (&**lr, &**rr)
                        {
                            if b >= a {
                                return Expression::Literal(Value::Bool(false));
                            }
                        }
                    }
                    (Expression::Equal(ll, lr), Expression::Equal(rl, rr))
                        if ll == rl && lr != rr =>
                    {
                        return Expression::Literal(Value::Bool(false));
                    }
                    _ => {} // Fall through to simple rules
                }
                apply_binary_rules!(l, r, Expression::And,
                    (_, Expression::Literal(Value::Bool(false))) | (Expression::Literal(Value::Bool(false)), _) => Expression::Literal(Value::Bool(false)),
                    (expr, Expression::Literal(Value::Bool(true))) | (Expression::Literal(Value::Bool(true)), expr) => expr,
                    (l_expr, r_expr) if l_expr == r_expr => l_expr,
                )
            }

            // --- Comparisons ---
            Expression::Equal(l, r) => apply_binary_rules!(l, r, Expression::Equal,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv == rv)),
                (Expression::Literal(Value::Bool(lv)), Expression::Literal(Value::Bool(rv))) => Expression::Literal(Value::Bool(lv == rv)),
            ),
            Expression::NotEqual(l, r) => apply_binary_rules!(l, r, Expression::NotEqual,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv != rv)),
                (Expression::Literal(Value::Bool(lv)), Expression::Literal(Value::Bool(rv))) => Expression::Literal(Value::Bool(lv != rv)),
            ),
            Expression::GreaterThan(l, r) => apply_binary_rules!(l, r, Expression::GreaterThan,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv > rv)),
            ),
            Expression::GreaterThanOrEqual(l, r) => {
                apply_binary_rules!(l, r, Expression::GreaterThanOrEqual,
                    (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv >= rv)),
                )
            }
            Expression::SmallerThan(l, r) => apply_binary_rules!(l, r, Expression::SmallerThan,
                (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv < rv)),
            ),
            Expression::SmallerThanOrEqual(l, r) => {
                apply_binary_rules!(l, r, Expression::SmallerThanOrEqual,
                    (Expression::Literal(Value::Number(lv)), Expression::Literal(Value::Number(rv))) => Expression::Literal(Value::Bool(lv <= rv)),
                )
            }

            // If no top-level rule matches, return the expression as is.
            other => other,
        }
    }
}
