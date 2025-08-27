use clap::Parser;
use hantei::prelude::*;
use serde::Deserialize;
use std::fs;
use std::time::Instant;

// --- JSON Deserialization Structs (Input Format Specific) ---
// These structs match the `flow.json` format and are only used here for conversion.

#[derive(Deserialize)]
struct RawRecipe {
    nodes: Vec<RawNode>,
    edges: Vec<RawEdge>,
}

#[derive(Deserialize)]
struct RawNode {
    id: String,
    data: RawNodeWrapper,
}

#[derive(Deserialize)]
struct RawNodeWrapper {
    #[serde(alias = "nodeData")]
    node_data: RawNodeData,
}

#[derive(Deserialize)]
struct RawNodeData {
    #[serde(alias = "realNodeType")]
    real_node_type: String,
    #[serde(alias = "realInputType")]
    real_input_type: Option<String>,
    values: Option<Vec<serde_json::Value>>,
    cases: Option<Vec<RawCase>>,
}

#[derive(Deserialize)]
struct RawCase {
    #[serde(alias = "caseId")]
    case_id: u32,
    #[serde(alias = "caseName")]
    case_name: String,
    #[serde(default, alias = "realCaseType")]
    real_case_type: Option<String>,
}

#[derive(Deserialize)]
struct RawEdge {
    source: String,
    #[serde(alias = "sourceHandle")]
    source_handle: String,
    target: String,
    #[serde(alias = "targetHandle")]
    target_handle: String,
}

#[derive(Deserialize)]
struct RawQuality {
    name: String,
    priority: i32,
}

// --- Converter Implementation ---
// This implements the conversion from the raw JSON model to Hantei's canonical FlowDefinition.

impl IntoFlow for RawRecipe {
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

/// A high-performance recipe compilation and evaluation engine CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the recipe flow JSON file
    recipe_path: String,
    /// Path to the qualities definition JSON file
    qualities_path: String,
    /// Optional path to the sample data JSON file for evaluation
    sample_data_path: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let total_start = Instant::now();

    // --- 1. File Loading ---
    let load_start = Instant::now();
    let recipe_json = fs::read_to_string(&cli.recipe_path).unwrap_or_else(|e| {
        exit_with_error(&format!(
            "Failed to read recipe file '{}': {}",
            &cli.recipe_path, e
        ))
    });
    let qualities_json = fs::read_to_string(&cli.qualities_path).unwrap_or_else(|e| {
        exit_with_error(&format!(
            "Failed to read qualities file '{}': {}",
            &cli.qualities_path, e
        ))
    });

    let sample_data = if let Some(data_path) = &cli.sample_data_path {
        SampleData::from_file(data_path).unwrap_or_else(|e| {
            exit_with_error(&format!(
                "Failed to load sample data from '{}': {}",
                data_path, e
            ))
        })
    } else {
        println!("No sample data file provided. Using default mock data.");
        SampleData::default()
    };
    let load_duration = load_start.elapsed();

    // --- 2. Parsing and Conversion ---
    let raw_recipe: RawRecipe = serde_json::from_str(&recipe_json)
        .unwrap_or_else(|e| exit_with_error(&format!("Failed to parse recipe JSON: {}", e)));
    let raw_qualities: Vec<RawQuality> = serde_json::from_str(&qualities_json)
        .unwrap_or_else(|e| exit_with_error(&format!("Failed to parse qualities JSON: {}", e)));

    let flow = raw_recipe
        .into_flow()
        .unwrap_or_else(|e| exit_with_error(&format!("Failed to convert recipe to flow: {}", e)));
    let qualities = raw_qualities
        .into_iter()
        .map(|q| Quality {
            name: q.name,
            priority: q.priority,
        })
        .collect();

    // --- 3. Compilation ---
    println!("\nStarting Hantei Recipe Compilation...");
    let compile_start = Instant::now();
    let compiler = Compiler::builder(flow, qualities).build();

    // **FIX:** `compile()` now only returns the `Vec`, not a tuple.
    let compiled_paths = compiler
        .compile()
        .unwrap_or_else(|e| exit_with_error(&format!("Compilation failed: {}", e)));
    let compile_duration = compile_start.elapsed();

    println!(
        "Compilation Successful! {} quality paths generated in {:?}",
        compiled_paths.len(),
        compile_duration
    );

    // --- 4. Evaluation ---
    println!("\nRunning Evaluation with Sample Data");
    let eval_start = Instant::now();
    let evaluator = Evaluator::new(compiled_paths);
    let result = evaluator
        .eval(sample_data.static_data(), sample_data.dynamic_data())
        .unwrap_or_else(|e| exit_with_error(&format!("Evaluation failed: {}", e)));
    let eval_duration = eval_start.elapsed();

    // --- 5. Results and Summary ---
    println!("\nEvaluation Finished!");
    if let Some(name) = result.quality_name {
        println!(
            "  -> Triggered Quality: {} (Priority {})",
            name,
            result.quality_priority.unwrap()
        );
        println!("  -> Reason: {}", result.reason);
    } else {
        println!("  -> No quality triggered");
    }

    let total_duration = total_start.elapsed();

    println!("\n--- Dataset Summary ---");
    println!("Static Fields: {}", sample_data.static_data().len());

    let dynamic_data = sample_data.dynamic_data();
    println!("Dynamic Event Types: {}", dynamic_data.len());
    let total_defects: usize = dynamic_data.values().map(|v| v.len()).sum();
    println!("Total Dynamic Events (Defects): {}", total_defects);

    println!("\n--- Performance Summary ---");
    println!("File Loading:      {:?}", load_duration);
    println!("AST Compilation:   {:?}", compile_duration);
    println!("Evaluation:        {:?}", eval_duration);
    println!("-----------------------------");
    println!("Total Execution:   {:?}", total_duration);
    println!();
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}
