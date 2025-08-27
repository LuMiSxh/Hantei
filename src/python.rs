use crate::compiler::Compiler;
use crate::evaluator::{EvaluationResult, Evaluator};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

impl<'py> IntoPyObject<'py> for EvaluationResult {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);

        // Handle Option fields - convert to Python None if None, otherwise convert the value
        match self.quality_name {
            Some(name) => dict.set_item("quality_name", name).unwrap(),
            None => dict.set_item("quality_name", py.None()).unwrap(),
        }

        match self.quality_priority {
            Some(priority) => dict.set_item("quality_priority", priority).unwrap(),
            None => dict.set_item("quality_priority", py.None()).unwrap(),
        }

        dict.set_item("reason", self.reason).unwrap();

        Ok(dict)
    }
}

/// A high-performance recipe compilation and evaluation engine.
///
/// This class compiles a recipe and quality definition upon initialization,
/// creating a highly optimized evaluation engine. The `evaluate` method can
/// then be called repeatedly with different data sets for fast, efficient
/// execution.
#[pyclass(name = "Hantei")]
struct HanteiPy {
    evaluator: Evaluator,
}

#[pymethods]
impl HanteiPy {
    /// Initializes and compiles the Hantei evaluator.
    ///
    /// This method parses the provided JSON strings, builds an Abstract
    /// Syntax Tree (AST) for each quality path, and applies optimizations.
    /// The resulting compiled engine is stored in the instance.
    ///
    /// Args:
    ///     recipe_json (str): A string containing the JSON definition of the
    ///         recipe flow, including nodes and edges.
    ///     qualities_json (str): A string containing the JSON array of
    ///         quality definitions, including names and priorities.
    ///
    /// Returns:
    ///     Hantei: An initialized instance of the Hantei evaluator.
    ///
    /// Raises:
    ///     ValueError: If there is an error during JSON parsing or recipe
    ///         compilation (e.g., malformed JSON, invalid node types,
    ///         missing nodes).
    #[new]
    fn new(recipe_json: &str, qualities_json: &str) -> PyResult<Self> {
        let compiler = Compiler::new(recipe_json, qualities_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let (_logical_repr, compiled_paths) = compiler
            .compile()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let evaluator = Evaluator::new(compiled_paths);
        Ok(HanteiPy { evaluator })
    }

    /// Evaluates the compiled recipe against the provided data.
    ///
    /// This method executes the pre-compiled logic against a set of static
    /// and dynamic data. It efficiently handles cross-product evaluation
    /// for dynamic events and returns the highest-priority quality that
    /// triggers.
    ///
    /// Args:
    ///     static_data (dict): A dictionary of static measurements, where keys
    ///         are measurement names (str) and values are numbers (float/int).
    ///     dynamic_data (dict): A dictionary of dynamic events. Keys are event
    ///         type names (str), and values are lists of dictionaries, where
    ///         each inner dictionary represents an event instance.
    ///
    /// Returns:
    ///     dict: A dictionary containing the evaluation result with three keys:
    ///         - "quality_name" (str | None): The name of the triggered quality.
    ///         - "quality_priority" (int | None): The priority of the triggered quality.
    ///         - "reason" (str): A human-readable trace of the logic that
    ///           led to the result.
    ///
    /// Raises:
    ///     RuntimeError: If an error occurs during evaluation, such as a
    ///         type mismatch in the expression logic or a required input
    ///         value not being found in the provided data.
    fn evaluate(
        &self,
        static_data_py: &Bound<'_, PyDict>,
        dynamic_data_py: &Bound<'_, PyDict>,
    ) -> PyResult<EvaluationResult> {
        // Extract data from Python dictionaries
        let static_data: HashMap<String, f64> = static_data_py.extract()?;
        let dynamic_data: HashMap<String, Vec<HashMap<String, f64>>> = dynamic_data_py.extract()?;

        let result = self
            .evaluator
            .eval(&static_data, &dynamic_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(result)
    }
}

/// A high-performance recipe compilation and evaluation engine.
///
/// This module provides Python bindings to the Hantei Rust library, allowing for
/// fast, ahead-of-time compilation of node-based decision trees and their
/// subsequent evaluation against runtime data.
#[pymodule]
fn hantei(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HanteiPy>()?;
    Ok(())
}
