use clap::{Parser, ValueEnum};
use hantei::backend::BackendChoice;
use hantei::prelude::*;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};

// --- JSON Deserialization Structs (Input Format Specific) ---
#[derive(Deserialize, Clone)]
struct RawRecipe {
    nodes: Vec<RawNode>,
    edges: Vec<RawEdge>,
}

#[derive(Deserialize, Clone)]
struct RawNode {
    id: String,
    data: RawNodeWrapper,
}

#[derive(Deserialize, Clone)]
struct RawNodeWrapper {
    #[serde(alias = "nodeData")]
    node_data: RawNodeData,
}

#[derive(Deserialize, Clone)]
struct RawNodeData {
    #[serde(alias = "realNodeType")]
    real_node_type: String,
    #[serde(alias = "realInputType")]
    real_input_type: Option<String>,
    values: Option<Vec<serde_json::Value>>,
    cases: Option<Vec<RawCase>>,
}

#[derive(Deserialize, Clone)]
struct RawCase {
    #[serde(alias = "caseId")]
    case_id: u32,
    #[serde(alias = "caseName")]
    case_name: String,
    #[serde(default, alias = "realCaseType")]
    real_case_type: Option<String>,
}

#[derive(Deserialize, Clone)]
struct RawEdge {
    source: String,
    #[serde(alias = "sourceHandle")]
    source_handle: String,
    target: String,
    #[serde(alias = "targetHandle")]
    target_handle: String,
}

#[derive(Deserialize, Clone)]
struct RawQuality {
    name: String,
    priority: i32,
}

/// Define a CLI-specific enum for clap to parse.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendCli {
    Interpreter,
    Bytecode,
}

// --- Converter Implementation ---
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
    recipe_path: Option<String>,
    /// Path to the qualities definition JSON file
    qualities_path: Option<String>,
    /// Optional path to the sample data JSON file for evaluation
    sample_data_path: Option<String>,

    /// The evaluation backend to use (ignored in benchmark mode)
    #[arg(short, long, value_enum)]
    backend: Option<BackendCli>,

    /// Run in interactive mode to be prompted for inputs
    #[arg(short = 'i', long, help = "Run in interactive 'human' mode")]
    human: bool,

    /// Run a benchmark comparing both backends for N iterations. e.g., --benchmark 100
    #[arg(
        short,
        long,
        help = "Run a benchmark for N iterations. e.g., --benchmark 100"
    )]
    benchmark: Option<Option<usize>>,
}

/// A struct to hold calculated statistics for a set of durations.
#[derive(Debug)]
struct Stats {
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    std_dev: Duration,
}

fn main() {
    let cli = Cli::parse();

    if cli.human {
        run_interactive();
    } else if let Some(iterations_opt) = cli.benchmark {
        let iterations = iterations_opt.unwrap_or(100);
        run_benchmark(cli, iterations);
    } else {
        run_non_interactive(cli);
    }
}

fn run_evaluation(
    recipe_path: String,
    qualities_path: String,
    sample_data_path: Option<String>,
    backend_choice: BackendChoice,
) {
    let total_start = Instant::now();

    // --- 1. File Loading ---
    let load_start = Instant::now();
    let recipe_json = fs::read_to_string(&recipe_path).unwrap_or_else(|e| {
        exit_with_error(&format!(
            "Failed to read recipe file '{}': {}",
            &recipe_path, e
        ))
    });
    let qualities_json = fs::read_to_string(&qualities_path).unwrap_or_else(|e| {
        exit_with_error(&format!(
            "Failed to read qualities file '{}': {}",
            &qualities_path, e
        ))
    });

    let sample_data = if let Some(data_path) = sample_data_path {
        SampleData::from_file(&data_path).unwrap_or_else(|e| {
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

    // --- 3. AST Compilation (Frontend) ---
    println!("\nStarting Hantei Recipe Compilation (AST Build)...");
    let compile_start = Instant::now();
    let compiler = Compiler::builder(flow, qualities).build();

    let compiled_paths = compiler
        .compile()
        .unwrap_or_else(|e| exit_with_error(&format!("AST compilation failed: {}", e)));
    let compile_duration = compile_start.elapsed();

    println!(
        "AST Compilation Successful! {} quality paths generated in {:?}",
        compiled_paths.len(),
        compile_duration
    );

    // --- 4. Backend Compilation ---
    println!("Compiling for backend: {:?}", backend_choice);
    let backend_start = Instant::now();

    let evaluator = Evaluator::new(backend_choice, compiled_paths)
        .unwrap_or_else(|e| exit_with_error(&format!("Backend compilation failed: {}", e)));
    let backend_duration = backend_start.elapsed();

    // --- 5. Evaluation ---
    println!("\nRunning Evaluation with Sample Data...");
    let eval_start = Instant::now();
    let result = evaluator
        .eval(sample_data.static_data(), sample_data.dynamic_data())
        .unwrap_or_else(|e| exit_with_error(&format!("Evaluation failed: {}", e)));
    let eval_duration = eval_start.elapsed();

    // --- 6. Results and Summary ---
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
    println!("File Loading:         {:?}", load_duration);
    println!("AST Compilation:      {:?}", compile_duration);
    println!("Backend Compilation:  {:?}", backend_duration);
    println!("Evaluation:           {:?}", eval_duration);
    println!("-----------------------------");
    println!("Total Execution:      {:?}", total_duration);
    println!("Backend Used:         {:?}", backend_choice);
    println!();
}

/// Runs the CLI in non-interactive mode.
fn run_non_interactive(cli: Cli) {
    let recipe_path = cli.recipe_path.unwrap_or_else(|| {
        exit_with_error("Recipe path is required in non-interactive mode.");
    });
    let qualities_path = cli.qualities_path.unwrap_or_else(|| {
        exit_with_error("Qualities path is required in non-interactive mode.");
    });
    let backend_cli = cli.backend.unwrap_or(BackendCli::Bytecode);
    let backend_choice = match backend_cli {
        BackendCli::Interpreter => BackendChoice::Interpreter,
        BackendCli::Bytecode => BackendChoice::Bytecode,
    };

    run_evaluation(
        recipe_path,
        qualities_path,
        cli.sample_data_path,
        backend_choice,
    );
}

/// Runs the CLI in an interactive, human-friendly mode.
fn run_interactive() {
    println!("--- Hantei Interactive Mode ---");

    let recipe_path = prompt_for_input("Enter recipe flow path", Some("data/flow.json"));
    let qualities_path = prompt_for_input("Enter qualities path", Some("data/qualities.json"));
    let sample_data_path_str = prompt_for_input(
        "Enter sample data path (optional)",
        Some("data/sample_data.json"),
    );

    let sample_data_path = if sample_data_path_str.is_empty() {
        None
    } else {
        Some(sample_data_path_str)
    };

    let mode_choice = loop {
        println!("\nPlease select a run mode:");
        println!("  1: Single Evaluation (with detailed result)");
        println!("  2: Benchmark (compare backends over many runs)");
        let choice_str = prompt_for_input("Enter choice", Some("1"));

        match choice_str.trim() {
            "1" => break "single",
            "2" => break "benchmark",
            _ => println!("Invalid choice. Please enter 1 or 2."),
        }
    };

    if mode_choice == "single" {
        let backend_choice = loop {
            println!("\nPlease select an evaluation backend:");
            println!("  1: Interpreter (slower, better debug traces)");
            println!("  2: Bytecode (faster execution)");
            let choice_str = prompt_for_input("Enter choice", Some("2"));

            match choice_str.trim() {
                "1" => break BackendChoice::Interpreter,
                "2" => break BackendChoice::Bytecode,
                _ => println!("Invalid choice. Please enter 1 or 2."),
            }
        };
        run_evaluation(
            recipe_path,
            qualities_path,
            sample_data_path,
            backend_choice,
        );
    } else {
        // mode_choice == "benchmark"
        let iterations_str = prompt_for_input("Enter number of iterations", Some("100"));
        let iterations = iterations_str.parse().unwrap_or(100);

        // Construct a Cli struct to pass to run_benchmark
        let cli_args = Cli {
            recipe_path: Some(recipe_path),
            qualities_path: Some(qualities_path),
            sample_data_path,
            backend: None,                     // Benchmark mode ignores this
            human: true,                       // We are in human mode
            benchmark: Some(Some(iterations)), // This is what triggers the mode
        };

        run_benchmark(cli_args, iterations);
    }
}

/// Runs the full pipeline in a loop for both backends to gather performance statistics.
fn run_benchmark(cli: Cli, iterations: usize) {
    let recipe_path = cli
        .recipe_path
        .unwrap_or_else(|| exit_with_error("Recipe path is required for benchmarking."));
    let qualities_path = cli
        .qualities_path
        .unwrap_or_else(|| exit_with_error("Qualities path is required for benchmarking."));

    println!("--- Hantei Benchmark Mode ---");
    println!("Recipe:     {}", recipe_path);
    println!("Qualities:  {}", qualities_path);
    println!("Iterations: {}", iterations);
    println!("--------------------------------");

    // --- Setup: Load data once ---
    let recipe_json = fs::read_to_string(&recipe_path)
        .unwrap_or_else(|e| exit_with_error(&format!("Failed to read recipe file: {}", e)));
    let qualities_json = fs::read_to_string(&qualities_path)
        .unwrap_or_else(|e| exit_with_error(&format!("Failed to read qualities file: {}", e)));
    let sample_data = if let Some(data_path) = cli.sample_data_path {
        println!("Data:       {}", data_path);
        SampleData::from_file(&data_path)
            .unwrap_or_else(|e| exit_with_error(&format!("Failed to load sample data: {}", e)))
    } else {
        println!("Data:       Default mock data");
        SampleData::default()
    };
    println!("--------------------------------");

    // --- Data Collection Vectors ---
    let mut ast_times_interp = Vec::with_capacity(iterations);
    let mut backend_times_interp = Vec::with_capacity(iterations);
    let mut eval_times_interp = Vec::with_capacity(iterations);

    let mut ast_times_bytecode = Vec::with_capacity(iterations);
    let mut backend_times_bytecode = Vec::with_capacity(iterations);
    let mut eval_times_bytecode = Vec::with_capacity(iterations);

    let backends_to_test = [BackendChoice::Interpreter, BackendChoice::Bytecode];
    let total_benchmark_start = Instant::now();

    for &backend_choice in &backends_to_test {
        println!("\nBenchmarking Backend: {:?}", backend_choice);

        for i in 0..iterations {
            print!("\r  Running iteration {}/{}...", i + 1, iterations);
            io::stdout().flush().unwrap();

            // These parsing steps are cheap and ensure a clean state for the compiler
            let raw_recipe: RawRecipe = serde_json::from_str(&recipe_json).unwrap();
            let raw_qualities: Vec<RawQuality> = serde_json::from_str(&qualities_json).unwrap();
            let flow = raw_recipe.into_flow().unwrap();
            let qualities: Vec<Quality> = raw_qualities
                .into_iter()
                .map(|q| Quality {
                    name: q.name,
                    priority: q.priority,
                })
                .collect();

            let ast_start = Instant::now();
            let compiler = Compiler::builder(flow, qualities).build();
            let compiled_paths = compiler.compile().unwrap();
            let ast_duration = ast_start.elapsed();

            let backend_start = Instant::now();
            let evaluator = Evaluator::new(backend_choice, compiled_paths).unwrap();
            let backend_duration = backend_start.elapsed();

            let eval_start = Instant::now();
            let _result = evaluator
                .eval(sample_data.static_data(), sample_data.dynamic_data())
                .unwrap();
            let eval_duration = eval_start.elapsed();

            match backend_choice {
                BackendChoice::Interpreter => {
                    ast_times_interp.push(ast_duration);
                    backend_times_interp.push(backend_duration);
                    eval_times_interp.push(eval_duration);
                }
                BackendChoice::Bytecode => {
                    ast_times_bytecode.push(ast_duration);
                    backend_times_bytecode.push(backend_duration);
                    eval_times_bytecode.push(eval_duration);
                }
            }
        }
    }
    println!(
        "\n\nBenchmark finished in {:?}.",
        total_benchmark_start.elapsed()
    );

    // --- Calculate and Display Statistics ---
    let ast_stats_interp = calculate_stats(&ast_times_interp).unwrap();
    let backend_stats_interp = calculate_stats(&backend_times_interp).unwrap();
    let eval_stats_interp = calculate_stats(&eval_times_interp).unwrap();

    let ast_stats_bytecode = calculate_stats(&ast_times_bytecode).unwrap();
    let backend_stats_bytecode = calculate_stats(&backend_times_bytecode).unwrap();
    let eval_stats_bytecode = calculate_stats(&eval_times_bytecode).unwrap();

    println!(
        "\n--- Performance Statistics ({} iterations) ---",
        iterations
    );
    println!(
        "| {:<21} | {:<12} | {:>12} | {:>12} | {:>12} | {:>12} | {:>12} |",
        "Stage", "Backend", "Min", "Max", "Mean", "Median", "Std Dev"
    );
    println!(
        "|-----------------------|--------------|--------------|--------------|--------------|--------------|--------------|"
    );
    print_stats_row("AST Compilation", "Interpreter", &ast_stats_interp);
    print_stats_row("", "Bytecode", &ast_stats_bytecode);
    println!(
        "|-----------------------|--------------|--------------|--------------|--------------|--------------|--------------|"
    );
    print_stats_row("Backend Compilation", "Interpreter", &backend_stats_interp);
    print_stats_row("", "Bytecode", &backend_stats_bytecode);
    println!(
        "|-----------------------|--------------|--------------|--------------|--------------|--------------|--------------|"
    );
    print_stats_row("Evaluation", "Interpreter", &eval_stats_interp);
    print_stats_row("", "Bytecode", &eval_stats_bytecode);

    println!("\n--- Summary ---");
    summarize_winner("AST Compilation", &ast_stats_interp, &ast_stats_bytecode);
    summarize_winner(
        "Backend Compilation",
        &backend_stats_interp,
        &backend_stats_bytecode,
    );
    summarize_winner("Evaluation", &eval_stats_interp, &eval_stats_bytecode);
    println!();
}

/// A helper function to prompt the user and read a line of input.
fn prompt_for_input(prompt_text: &str, default: Option<&str>) -> String {
    let mut line = String::new();
    let default_prompt = default.map_or("".to_string(), |d| format!(" [default: {}]", d));

    print!("> {}{}: ", prompt_text, default_prompt);
    io::stdout().flush().unwrap();

    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    let trimmed = line.trim().to_string();

    if trimmed.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        trimmed
    }
}

/// Calculates statistics for a slice of Durations.
fn calculate_stats(durations: &[Duration]) -> Option<Stats> {
    if durations.is_empty() {
        return None;
    }

    let mut sorted_durations = durations.to_vec();
    sorted_durations.sort();

    let min = *sorted_durations.first().unwrap();
    let max = *sorted_durations.last().unwrap();

    let sum: Duration = sorted_durations.iter().sum();
    let count = durations.len() as u32;
    let mean = sum / count;

    let median = if count % 2 == 0 {
        (sorted_durations[count as usize / 2 - 1] + sorted_durations[count as usize / 2]) / 2
    } else {
        sorted_durations[count as usize / 2]
    };

    let mean_secs = mean.as_secs_f64();
    let variance = sorted_durations
        .iter()
        .map(|d| {
            let diff = d.as_secs_f64() - mean_secs;
            diff * diff
        })
        .sum::<f64>()
        / count as f64;

    let std_dev = Duration::from_secs_f64(variance.sqrt());

    Some(Stats {
        min,
        max,
        mean,
        median,
        std_dev,
    })
}

/// Helper to print a formatted row in the statistics table.
fn print_stats_row(name: &str, backend: &str, stats: &Stats) {
    println!(
        "| {:<21} | {:<12} | {:>12.4?} | {:>12.4?} | {:>12.4?} | {:>12.4?} | {:>12.4?} |",
        name, backend, stats.min, stats.max, stats.mean, stats.median, stats.std_dev
    );
}

/// Prints a summary line comparing the mean times of the two backends for a stage.
fn summarize_winner(stage: &str, stats_interp: &Stats, stats_bytecode: &Stats) {
    let mean_interp = stats_interp.mean.as_nanos();
    let mean_bytecode = stats_bytecode.mean.as_nanos();

    if mean_bytecode < mean_interp {
        let diff = mean_interp - mean_bytecode;
        let percentage = (diff as f64 / mean_interp as f64) * 100.0;
        println!(
            "- {:<21}: Bytecode was faster by {:?} ({:.2}%) on average.",
            stage,
            Duration::from_nanos(diff as u64),
            percentage
        );
    } else {
        let diff = mean_bytecode - mean_interp;
        let percentage = (diff as f64 / mean_bytecode as f64) * 100.0;
        println!(
            "- {:<21}: Interpreter was faster by {:?} ({:.2}%) on average.",
            stage,
            Duration::from_nanos(diff as u64),
            percentage
        );
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("\nError: {}", message);
    std::process::exit(1);
}
