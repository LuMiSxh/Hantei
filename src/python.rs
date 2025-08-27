use crate::compiler::Compiler;
use crate::error::RecipeConversionError;
use crate::evaluator::{EvaluationResult, Evaluator};
use crate::recipe::{
    DataFieldDefinition, FlowDefinition, FlowEdgeDefinition, FlowNodeDefinition, IntoFlow, Quality,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

// --- JSON Deserialization Structs (Input Format Specific) ---
// These structs are private to the Python module and are used to parse the
// user-provided JSON strings before converting them to Hantei's internal format.
mod json_models {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct RawRecipe {
        pub(super) nodes: Vec<RawNode>,
        pub(super) edges: Vec<RawEdge>,
    }

    #[derive(Deserialize)]
    pub(super) struct RawNode {
        pub(super) id: String,
        pub(super) data: RawNodeWrapper,
    }

    #[derive(Deserialize)]
    pub(super) struct RawNodeWrapper {
        #[serde(alias = "nodeData")]
        pub(super) node_data: RawNodeData,
    }

    #[derive(Deserialize)]
    pub(super) struct RawNodeData {
        #[serde(alias = "realNodeType")]
        pub(super) real_node_type: String,
        #[serde(alias = "realInputType")]
        pub(super) real_input_type: Option<String>,
        pub(super) values: Option<Vec<serde_json::Value>>,
        pub(super) cases: Option<Vec<RawCase>>,
    }

    #[derive(Deserialize)]
    pub(super) struct RawCase {
        #[serde(alias = "caseId")]
        pub(super) case_id: u32,
        #[serde(alias = "caseName")]
        pub(super) case_name: String,
        #[serde(default, alias = "realCaseType")]
        pub(super) real_case_type: Option<String>,
    }

    #[derive(Deserialize)]
    pub(super) struct RawEdge {
        pub(super) source: String,
        #[serde(alias = "sourceHandle")]
        pub(super) source_handle: String,
        pub(super) target: String,
        #[serde(alias = "targetHandle")]
        pub(super) target_handle: String,
    }

    #[derive(Deserialize)]
    pub(super) struct RawQuality {
        pub(super) name: String,
        pub(super) priority: i32,
    }
}

// --- Converter Implementation ---
// Implements the conversion from the raw JSON model to Hantei's canonical FlowDefinition.
impl IntoFlow for json_models::RawRecipe {
    fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> {
        let nodes = self
            .nodes
            .into_iter()
            .map(|raw_node| FlowNodeDefinition {
                id: raw_node.id,
                operation_type: raw_node.data.node_data.real_node_type,
                input_type: raw_node.data.node_data.real_input_type,
                literal_values: raw_node.data.node_data.values,
                data_fields: raw_node.data.node_data.cases.map(|cases| {
                    cases
                        .into_iter()
                        .map(|c| DataFieldDefinition {
                            id: c.case_id,
                            name: c.case_name,
                            data_type: c.real_case_type,
                        })
                        .collect()
                }),
            })
            .collect();

        let edges = self
            .edges
            .into_iter()
            .map(|raw_edge| FlowEdgeDefinition {
                source: raw_edge.source,
                source_handle: raw_edge.source_handle,
                target: raw_edge.target,
                target_handle: raw_edge.target_handle,
            })
            .collect();

        Ok(FlowDefinition { nodes, edges })
    }
}

impl<'py> IntoPyObject<'py> for EvaluationResult {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new_bound(py);

        match self.quality_name {
            Some(name) => dict.set_item("quality_name", name).unwrap(),
            None => dict.set_item("quality_name", py.None()).unwrap(),
        }
        match self.quality_priority {
            Some(priority) => dict.set_item("quality_priority", priority).unwrap(),
            None => dict.set_item("quality_priority", py.None()).unwrap(),
        }
        dict.set_item("reason", self.reason).unwrap();
        dict.into()
    }
}

/// A high-performance recipe compilation and evaluation engine.
#[pyclass(name = "Hantei")]
struct HanteiPy {
    evaluator: Evaluator,
}

#[pymethods]
impl HanteiPy {
    /// Initializes and compiles the Hantei evaluator from JSON strings.
    #[new]
    fn new(recipe_json: &str, qualities_json: &str) -> PyResult<Self> {
        // 1. Parse the raw JSON strings into our temporary models.
        let raw_recipe: json_models::RawRecipe =
            serde_json::from_str(recipe_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Recipe JSON parsing error: {}",
                    e
                ))
            })?;
        let raw_qualities: Vec<json_models::RawQuality> = serde_json::from_str(qualities_json)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Qualities JSON parsing error: {}",
                    e
                ))
            })?;

        // 2. Convert the raw models into Hantei's canonical, internal data structures.
        let flow = raw_recipe
            .into_flow()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let qualities = raw_qualities
            .into_iter()
            .map(|q| Quality {
                name: q.name,
                priority: q.priority,
            })
            .collect();

        // 3. Use the builder to create and run the compiler with the canonical data.
        let compiler = Compiler::builder(flow, qualities).build();
        let compiled_paths = compiler
            .compile()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        // 4. Create the evaluator with the optimized ASTs.
        let evaluator = Evaluator::new(compiled_paths);
        Ok(HanteiPy { evaluator })
    }

    /// Evaluates the compiled recipe against the provided data.
    fn evaluate(
        &self,
        static_data_py: &Bound<'_, PyDict>,
        dynamic_data_py: &Bound<'_, PyDict>,
    ) -> PyResult<EvaluationResult> {
        let static_data: HashMap<String, f64> = static_data_py.extract()?;
        let dynamic_data: HashMap<String, Vec<HashMap<String, f64>>> = dynamic_data_py.extract()?;

        let result = self
            .evaluator
            .eval(&static_data, &dynamic_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(result)
    }
}

/// Python bindings for the Hantei compilation and evaluation engine.
#[pymodule]
fn hantei(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HanteiPy>()?;
    Ok(())
}
