use super::engine::AstEngine;
use crate::ast::{EvaluationTrace, Expression, Value};
use crate::error::EvaluationError;
use ahash::AHashMap;
use std::collections::HashSet;

/// Handles the cross-product evaluation for expressions with dynamic inputs.
pub(super) struct DynamicEvaluator<'a> {
    ast: &'a Expression,
    event_types: Vec<String>,
    static_data: &'a AHashMap<String, f64>,
    dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
}

impl<'a> DynamicEvaluator<'a> {
    pub(super) fn new(
        ast: &'a Expression,
        required_events: &HashSet<String>,
        static_data: &'a AHashMap<String, f64>,
        dynamic_data: &'a AHashMap<String, Vec<AHashMap<String, f64>>>,
    ) -> Self {
        let mut event_types: Vec<String> = required_events.iter().cloned().collect();

        // Sort event types by the number of instances, from fewest to most.
        // This prunes the search tree much faster by failing on smaller sets first.
        event_types.sort_by_key(|event_type| dynamic_data.get(event_type).map_or(0, |v| v.len()));

        Self {
            ast,
            event_types,
            static_data,
            dynamic_data,
        }
    }

    /// Searches for any combination of dynamic events that makes the AST true.
    /// Returns the first successful trace found.
    pub(super) fn evaluate(&self) -> Result<Option<EvaluationTrace>, EvaluationError> {
        // If any part of the logic that depends on a single event type is impossible
        // to satisfy, we can exit immediately.
        if !self.pre_filter()? {
            return Ok(None);
        }

        let mut context = AHashMap::new();
        self.evaluate_recursive(&self.event_types, &mut context)
    }

    /// Checks if single-event sub-expressions can ever be true.
    fn pre_filter(&self) -> Result<bool, EvaluationError> {
        let mut sub_expressions = Vec::new();
        self.collect_single_event_sub_expressions(self.ast, &mut sub_expressions);

        for (sub_expr, event_type) in sub_expressions {
            let event_instances = self.dynamic_data.get(&event_type);

            // If the required event type has no data, the sub-expression can't be satisfied.
            if event_instances.is_none() || event_instances.unwrap().is_empty() {
                return Ok(false);
            }

            let mut can_be_true = false;
            for instance in event_instances.unwrap() {
                let mut context = AHashMap::new();
                context.insert(event_type.clone(), instance);
                let engine = AstEngine::new(sub_expr, self.static_data, &context);
                let trace = engine.evaluate()?;
                if let Value::Bool(true) = trace.get_outcome() {
                    can_be_true = true;
                    break; // Found at least one satisfying instance.
                }
            }

            // If after checking all instances, none made the sub-expression true,
            // then the entire quality path is impossible.
            if !can_be_true {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Traverses the AST to find branches that depend on only one dynamic event.
    fn collect_single_event_sub_expressions<'e>(
        &self,
        expr: &'e Expression,
        collection: &mut Vec<(&'e Expression, String)>,
    ) {
        let mut required_events = HashSet::new();
        expr.get_required_events(&mut required_events);

        if required_events.len() == 1 {
            let event_type = required_events.into_iter().next().unwrap();
            collection.push((expr, event_type));
        } else if required_events.len() > 1 {
            // Recurse into children only if there's a mix of events.
            match expr {
                Expression::And(l, r) | Expression::Or(l, r) => {
                    self.collect_single_event_sub_expressions(l, collection);
                    self.collect_single_event_sub_expressions(r, collection);
                }
                _ => {}
            }
        }
    }

    fn evaluate_recursive(
        &self,
        remaining_events: &[String],
        context: &mut AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Result<Option<EvaluationTrace>, EvaluationError> {
        // Base case: all event types have been assigned a context, evaluate the AST.
        if remaining_events.is_empty() {
            let engine = AstEngine::new(self.ast, self.static_data, context);
            let trace = engine.evaluate()?;
            if let Value::Bool(true) = trace.get_outcome() {
                return Ok(Some(trace));
            }
            return Ok(None);
        }

        // Recursive step: iterate through instances of the current event type.
        let current_event_type = &remaining_events[0];
        let next_event_types = &remaining_events[1..];

        // If an event type has no instances, we can't find a match.
        if let Some(instances) = self.dynamic_data.get(current_event_type) {
            if instances.is_empty() {
                // An empty list means this path cannot succeed.
                return Ok(None);
            }
            for instance in instances {
                context.insert(current_event_type.clone(), instance);
                if let Some(trace) = self.evaluate_recursive(next_event_types, context)? {
                    // A match was found in a deeper recursive call, so we stop and return it.
                    return Ok(Some(trace));
                }
            }
        } else {
            // If the required event type doesn't exist in the data, it's a non-match for any combination.
            return Ok(None);
        }

        Ok(None)
    }
}
