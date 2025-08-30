use crate::backend::BackendChoice;
use crate::compiler::Compiler;
use crate::error::RecipeConversionError;
use crate::evaluator::Evaluator;
use crate::interpreter::EvaluationResult as RustEvaluationResult;
use crate::recipe::{
    DataFieldDefinition, FlowDefinition, FlowEdgeDefinition, FlowNodeDefinition, IntoFlow, Quality,
};
use ahash::AHashMap;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

// --- JSON Deserialization Structs ---
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

#[pyclass(name = "EvaluationResult")]
#[derive(Debug, Clone)]
struct PyEvaluationResult {
    #[pyo3(get)]
    quality_name: Option<String>,
    #[pyo3(get)]
    quality_priority: Option<i32>,
    #[pyo3(get)]
    reason: String,
}

#[pymethods]
impl PyEvaluationResult {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<RustEvaluationResult> for PyEvaluationResult {
    fn from(res: RustEvaluationResult) -> Self {
        PyEvaluationResult {
            quality_name: res.quality_name,
            quality_priority: res.quality_priority,
            reason: res.reason,
        }
    }
}

/// A high-performance recipe compilation and evaluation engine.
#[pyclass(name = "Hantei")]
struct HanteiPy {
    evaluator: Evaluator,
}

#[pymethods]
impl HanteiPy {
    #[new]
    #[pyo3(signature = (recipe_json, qualities_json, backend="bytecode"))]
    fn new(recipe_json: &str, qualities_json: &str, backend: &str) -> PyResult<Self> {
        let raw_recipe: json_models::RawRecipe = serde_json::from_str(recipe_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let raw_qualities: Vec<json_models::RawQuality> = serde_json::from_str(qualities_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

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

        let compiler = Compiler::builder(flow, qualities).build();
        let compiled_paths = compiler
            .compile()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        // --- FIX 4: Use the new Evaluator API ---
        let choice = match backend {
            "interpreter" => BackendChoice::Interpreter,
            "bytecode" => BackendChoice::Bytecode,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid backend. Choose from 'interpreter' or 'bytecode'.",
                ));
            }
        };

        let evaluator = Evaluator::new(choice, compiled_paths)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(HanteiPy { evaluator })
    }

    /// Evaluates the compiled recipe against the provided data.
    fn evaluate(
        &self,
        static_data_py: &Bound<'_, PyDict>,
        dynamic_data_py: &Bound<'_, PyDict>,
    ) -> PyResult<PyEvaluationResult> {
        let static_data_std: HashMap<String, f64> = static_data_py.extract()?;
        let dynamic_data_std: HashMap<String, Vec<HashMap<String, f64>>> =
            dynamic_data_py.extract()?;

        let static_data: AHashMap<String, f64> = static_data_std.into_iter().collect();
        let dynamic_data: AHashMap<String, Vec<AHashMap<String, f64>>> = dynamic_data_std
            .into_iter()
            .map(|(key, vec_of_maps)| {
                (
                    key,
                    vec_of_maps
                        .into_iter()
                        .map(|std_map| std_map.into_iter().collect())
                        .collect(),
                )
            })
            .collect();

        let result = self
            .evaluator
            .eval(&static_data, &dynamic_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        // Convert the internal Rust result into the Python class and return
        Ok(result.into())
    }
}

#[pymodule]
fn hantei(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HanteiPy>()?;
    m.add_class::<PyEvaluationResult>()?;
    Ok(())
}
