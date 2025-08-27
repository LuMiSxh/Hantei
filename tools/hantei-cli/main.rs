use clap::Parser;
use hantei::{Compiler, Evaluator, SampleData};
use std::fs;
use std::time::Instant;

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

    const TMP_DIR: &str = "tmp";
    if let Err(e) = fs::create_dir_all(TMP_DIR) {
        eprintln!("Failed to create tmp directory: {}", e);
        std::process::exit(1);
    }

    let load_start = Instant::now();
    let recipe_json = fs::read_to_string(&cli.recipe_path).unwrap_or_else(|e| {
        eprintln!("Failed to read recipe file '{}': {}", &cli.recipe_path, e);
        std::process::exit(1);
    });
    let qualities_json = fs::read_to_string(&cli.qualities_path).unwrap_or_else(|e| {
        eprintln!(
            "Failed to read qualities file '{}': {}",
            &cli.qualities_path, e
        );
        std::process::exit(1);
    });
    let sample_data = if let Some(data_path) = &cli.sample_data_path {
        SampleData::from_file(data_path).unwrap_or_else(|e| {
            eprintln!("Failed to load sample data from '{}': {}", data_path, e);
            std::process::exit(1);
        })
    } else {
        println!("No sample data file provided. Using default mock data.");
        SampleData::default()
    };
    let load_duration = load_start.elapsed();

    println!("\nStarting Hantei Recipe Compilation...");
    let compile_start = Instant::now();
    let compiler = Compiler::new(&recipe_json, &qualities_json).unwrap_or_else(|e| {
        eprintln!("Failed to create compiler: {}", e);
        std::process::exit(1);
    });

    // Pass the CLI flag to the compiler
    let (_logical_repr, compiled_paths) = compiler.compile().unwrap_or_else(|e| {
        eprintln!("Compilation failed: {}", e);
        std::process::exit(1);
    });
    let compile_duration = compile_start.elapsed();

    println!(
        "Compilation Successful! {} quality paths generated in {:?}",
        compiled_paths.len(),
        compile_duration
    );

    println!("\nRunning Evaluation with Sample Data");
    let eval_start = Instant::now();
    let evaluator = Evaluator::new(compiled_paths);
    let result = evaluator
        .eval(sample_data.static_data(), sample_data.dynamic_data())
        .unwrap_or_else(|e| {
            eprintln!("Evaluation failed: {}", e);
            std::process::exit(1);
        });
    let eval_duration = eval_start.elapsed();

    // Print Quality, Reason
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

    // Print Dataset Summary
    println!("\n--- Dataset Summary ---");
    println!("Static Fields: {}", sample_data.static_data().len());

    let dynamic_data = sample_data.dynamic_data();
    println!("Dynamic Event Types: {}", dynamic_data.len());

    let mut total_defects = 0;
    let mut sorted_events: Vec<_> = dynamic_data.keys().collect();
    sorted_events.sort(); // Sort keys alphabetically for consistent output

    for event_type in sorted_events {
        if let Some(events) = dynamic_data.get(event_type) {
            println!("  - '{}': {} instances", event_type, events.len());
            total_defects += events.len();
        }
    }
    println!("Total Dynamic Events (Defects): {}", total_defects);

    println!("\n--- Performance Summary ---");
    println!("File Loading:      {:?}", load_duration);
    println!("AST Compilation:   {:?}", compile_duration);
    println!("Evaluation:        {:?}", eval_duration);
    println!("-----------------------------");
    println!("Total Execution:   {:?}", total_duration);
    println!();
}
