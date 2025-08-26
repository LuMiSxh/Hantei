use hantei::compiler::Compiler;
use hantei::evaluator::Evaluator;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Deserialize, Debug)]
struct SampleData {
    static_data: HashMap<String, f64>,
    dynamic_data: HashMap<String, Vec<HashMap<String, f64>>>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    const TMP_DIR: &str = "tmp";
    fs::create_dir_all(TMP_DIR).expect("Failed to create tmp directory");
    log::info!("Created output directory at '{}'", TMP_DIR);

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        log::error!(
            "Usage: cargo run -- <path/to/recipe.json> <path/to/qualities.json> [path/to/sample_data.json]"
        );
        panic!("Invalid number of arguments provided.");
    }
    let recipe_path = &args[1];
    let qualities_path = &args[2];

    log::info!("Loading recipe from: {}", recipe_path);
    log::info!("Loading qualities from: {}", qualities_path);

    let recipe_json =
        fs::read_to_string(recipe_path).expect("Failed to read the recipe JSON file.");
    let qualities_json =
        fs::read_to_string(qualities_path).expect("Failed to read the qualities JSON file.");

    let (static_data, dynamic_data);

    if let Some(data_path) = args.get(3) {
        log::info!("Loading sample data from: {}", data_path);
        let data_json =
            fs::read_to_string(data_path).expect("Failed to read the sample data JSON file.");
        let sample_data: SampleData =
            serde_json::from_str(&data_json).expect("Failed to parse sample data JSON.");
        static_data = sample_data.static_data;
        dynamic_data = sample_data.dynamic_data;
    } else {
        log::info!("No sample data file provided. Using default mock data.");
        // Fallback to default mock data if no file is given
        let mut s_data = HashMap::new();
        s_data.insert("Leading width".to_string(), 1970.0);
        s_data.insert("Trailing width".to_string(), 1965.0);
        static_data = s_data;

        let mut d_data = HashMap::new();
        let mut hole_event = HashMap::new();
        hole_event.insert("Diameter".to_string(), 30.0);
        d_data.insert("hole".to_string(), vec![hole_event]);
        dynamic_data = d_data;
    }

    log::info!("Starting Hantei Recipe Compilation...");

    let compiler = Compiler::new(&recipe_json, &qualities_json).expect("Failed to create compiler");
    let (logical_repr, compiled_paths) = compiler.compile().expect("Failed to compile recipe");
    let logical_path = format!("{}/logical_connections.txt", TMP_DIR);
    fs::write(&logical_path, logical_repr).expect("Unable to write logical representation to file");
    log::info!("  -> Wrote logical representation to '{}'", logical_path);

    log::info!(
        "Compilation Successful! {} quality paths generated.\n",
        compiled_paths.len()
    );

    let evaluator = Evaluator::new(compiled_paths);
    log::info!("Evaluator created and ready.");

    log::info!("Running Evaluation with Sample Data");
    log::debug!("Static Data: {:?}", static_data);
    log::debug!("Dynamic Data: {:?}", dynamic_data);

    let result = evaluator
        .eval(&static_data, &dynamic_data)
        .expect("Evaluation failed");

    log::info!("Evaluation Finished!");
    log::info!("  -> Result: {:?}", result);
}
