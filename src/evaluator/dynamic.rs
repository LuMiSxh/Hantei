use super::engine::AstEngine;
use crate::ast::{EvaluationTrace, Expression, Value};
use crate::error::EvaluationError;
use std::collections::{HashMap, HashSet};

/// Handles the cross-product evaluation for expressions with dynamic inputs.
pub(super) struct DynamicEvaluator<'a> {
    ast: &'a Expression,
    event_types: Vec<String>,
    static_data: &'a HashMap<String, f64>,
    dynamic_data: &'a HashMap<String, Vec<HashMap<String, f64>>>,
}

impl<'a> DynamicEvaluator<'a> {
    pub(super) fn new(
        ast: &'a Expression,
        required_events: &HashSet<String>,
        static_data: &'a HashMap<String, f64>,
        dynamic_data: &'a HashMap<String, Vec<HashMap<String, f64>>>,
    ) -> Self {
        Self {
            ast,
            event_types: required_events.iter().cloned().collect(),
            static_data,
            dynamic_data,
        }
    }

    /// Searches for any combination of dynamic events that makes the AST true.
    /// Returns the first successful trace found.
    pub(super) fn evaluate(&self) -> Result<Option<EvaluationTrace>, EvaluationError> {
        let mut context = HashMap::new();
        self.evaluate_recursive(&self.event_types, &mut context)
    }

    fn evaluate_recursive(
        &self,
        remaining_events: &[String],
        context: &mut HashMap<String, &'a HashMap<String, f64>>,
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
