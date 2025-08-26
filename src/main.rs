use hantei::{Compiler, Evaluator, SampleData};
use std::env;
use std::fs;

fn main() {
    // Create output directory
    const TMP_DIR: &str = "tmp";
    if let Err(e) = fs::create_dir_all(TMP_DIR) {
        eprintln!("Failed to create tmp directory: {}", e);
        std::process::exit(1);
    }
    println!("Created output directory at '{}'", TMP_DIR);

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!(
            "Usage: cargo run -- <path/to/recipe.json> <path/to/qualities.json> [path/to/sample_data.json]"
        );
        std::process::exit(1);
    }

    let recipe_path = &args[1];
    let qualities_path = &args[2];
    let sample_data_path = args.get(3);

    println!("Loading recipe from: {}", recipe_path);
    println!("Loading qualities from: {}", qualities_path);

    // Load input files
    let recipe_json = match fs::read_to_string(recipe_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read recipe file '{}': {}", recipe_path, e);
            std::process::exit(1);
        }
    };

    let qualities_json = match fs::read_to_string(qualities_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read qualities file '{}': {}", qualities_path, e);
            std::process::exit(1);
        }
    };

    // Load sample data
    let sample_data = if let Some(data_path) = sample_data_path {
        println!("Loading sample data from: {}", data_path);
        match SampleData::from_file(data_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to load sample data from '{}': {}", data_path, e);
                std::process::exit(1);
            }
        }
    } else {
        println!("No sample data file provided. Using default mock data.");
        SampleData::default()
    };

    // Compilation phase
    println!("\nStarting Hantei Recipe Compilation...");

    let compiler = match Compiler::new(&recipe_json, &qualities_json) {
        Ok(compiler) => compiler,
        Err(e) => {
            eprintln!("Failed to create compiler: {}", e);
            std::process::exit(1);
        }
    };

    let (logical_repr, compiled_paths) = match compiler.compile() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    };

    // Write logical representation to file
    let logical_path = format!("{}/logical_connections.txt", TMP_DIR);
    if let Err(e) = fs::write(&logical_path, logical_repr) {
        eprintln!("Failed to write logical representation: {}", e);
        std::process::exit(1);
    }
    println!("  -> Wrote logical representation to '{}'", logical_path);

    println!(
        "Compilation Successful! {} quality paths generated.",
        compiled_paths.len()
    );

    // Print compiled quality information
    for (priority, name, _) in &compiled_paths {
        println!("  -> Quality '{}' (Priority {}) compiled", name, priority);
    }

    // Evaluation phase
    println!("\nRunning Evaluation with Sample Data");
    println!(
        "Static data keys: {:?}",
        sample_data.static_data().keys().collect::<Vec<_>>()
    );
    println!(
        "Dynamic data keys: {:?}",
        sample_data.dynamic_data().keys().collect::<Vec<_>>()
    );

    let evaluator = Evaluator::new(compiled_paths);
    let result = match evaluator.eval(sample_data.static_data(), sample_data.dynamic_data()) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Evaluation failed: {}", e);
            std::process::exit(1);
        }
    };

    // Display results
    println!("\nEvaluation Finished!");
    match result.quality_name {
        Some(name) => {
            println!(
                "  -> Triggered Quality: {} (Priority {})",
                name,
                result.quality_priority.unwrap()
            );
            println!("  -> Reason: {}", result.reason);
        }
        None => {
            println!("  -> No quality triggered");
            println!("  -> Reason: {}", result.reason);
        }
    }
    println!();
}
