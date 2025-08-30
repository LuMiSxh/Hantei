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
        let mut context = AHashMap::new();
        self.evaluate_recursive(&self.event_types, &mut context)
    }

    /// Recursively evaluates the AST for all combinations of dynamic event instances.
    /// Returns the first successful trace found.
    fn evaluate_recursive(
        &self,
        remaining_events: &[String],
        context: &mut AHashMap<String, &'a AHashMap<String, f64>>,
    ) -> Result<Option<EvaluationTrace>, EvaluationError> {
        if remaining_events.is_empty() {
            let engine = AstEngine::new(self.ast, self.static_data, context);
            let trace = engine.evaluate()?;
            if let Value::Bool(true) = trace.get_outcome() {
                return Ok(Some(trace));
            }
            return Ok(None);
        }

        let current_event_type = &remaining_events[0];
        let next_event_types = &remaining_events[1..];

        if let Some(instances) = self.dynamic_data.get(current_event_type) {
            if instances.is_empty() {
                return Ok(None);
            }
            for instance in instances {
                context.insert(current_event_type.clone(), instance);
                if let Some(trace) = self.evaluate_recursive(next_event_types, context)? {
                    return Ok(Some(trace));
                }
            }
        } else {
            return Ok(None);
        }

        Ok(None)
    }
}
