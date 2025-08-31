use ahash::AHashMap;
use clap::Parser;
use hantei::data::SampleData;
use rand::{Rng, rngs::ThreadRng, thread_rng};
use std::fs;

/// A CLI tool to generate sample data for the Hantei evaluator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to write the generated JSON file to
    #[arg(short, long, default_value = "generated_data.json")]
    output: String,

    /// The minimum number of instances to generate for each event type
    #[arg(long, default_value_t = 0)]
    min: usize,

    /// The maximum number of instances to generate for each event type
    #[arg(long, default_value_t = 20)]
    max: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut rng = thread_rng();

    // Add validation to ensure min is not greater than max
    if cli.min > cli.max {
        eprintln!(
            "Error: --min ({}) cannot be greater than --max ({})",
            cli.min, cli.max
        );
        std::process::exit(1);
    }

    println!(
        "Generating new test data (event instances per type: {} to {})...",
        cli.min, cli.max
    );

    let static_data = generate_static_data(&mut rng);
    // Pass the min/max values to the dynamic data generator
    let dynamic_data = generate_dynamic_data(&mut rng, cli.min, cli.max);

    let sample_data = SampleData {
        static_data,
        dynamic_data,
    };

    let json_output = serde_json::to_string_pretty(&sample_data)?;
    fs::write(&cli.output, json_output)?;

    println!(
        "Successfully generated and saved test data to '{}'",
        cli.output
    );

    Ok(())
}

/// Generates the static "veneer" data.
fn generate_static_data(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    // ... (this function is unchanged)
    let mut data = AHashMap::new();
    data.insert("Leading width".to_string(), rng.gen_range(1800.0..2200.0));
    data.insert("Trailing width".to_string(), rng.gen_range(1800.0..2200.0));
    data.insert("Upper length".to_string(), rng.gen_range(2000.0..2500.0));
    data.insert("Lower length".to_string(), rng.gen_range(2000.0..2500.0));
    data.insert("Area".to_string(), rng.gen_range(4_000_000.0..5_000_000.0));
    data.insert("Angle".to_string(), rng.gen_range(89.0..91.0));
    data.insert("Humidity".to_string(), rng.gen_range(5.0..10.0));
    data.insert("Humidity peak".to_string(), rng.gen_range(8.0..15.0));
    println!("-> Generated static data.");
    data
}

/// Generates the dynamic event data using the provided min/max range.
fn generate_dynamic_data(
    rng: &mut ThreadRng,
    min_events: usize,
    max_events: usize,
) -> AHashMap<String, Vec<AHashMap<String, f64>>> {
    let mut data = AHashMap::new();

    // We now just define the event type and its field generator.
    // The number of instances will be determined by the CLI arguments.
    let event_configs: Vec<(&str, fn(&mut ThreadRng) -> AHashMap<String, f64>)> = vec![
        ("hole", generate_hole_event),
        ("tear", generate_tear_event),
        ("inner_tear", generate_inner_tear_event),
        ("healthy_branch", generate_branch_event),
        ("black_branch", generate_branch_event),
        ("bark", generate_bark_event),
        // Add other events that should be randomized here
    ];

    // These events will always be empty, ignoring the min/max flags.
    let fixed_empty_events = [
        "clipping_strip",
        "branch",
        "discoloration",
        "stipple",
        "rotary_growth",
        "missing_edge",
        "white_rot",
        "brown_rot",
    ];

    // Generate randomized events
    for (name, generator_fn) in event_configs {
        let count = rng.gen_range(min_events..=max_events);
        let events = (0..count).map(|_| generator_fn(rng)).collect();
        data.insert(name.to_string(), events);
        if count > 0 {
            println!("-> Generated {} instance(s) of '{}'.", count, name);
        }
    }

    // Ensure fixed events are present but empty
    for name in fixed_empty_events {
        data.insert(name.to_string(), vec![]);
    }

    data
}

// --- Field Generator Functions for Each Event Type ---

fn generate_hole_event(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    let mut fields = AHashMap::new();
    fields.insert("Diameter".to_string(), rng.gen_range(5.0..100.0));
    fields.insert("Length".to_string(), rng.gen_range(10.0..150.0));
    fields.insert("Area".to_string(), rng.gen_range(50.0..5000.0));
    fields
}

fn generate_tear_event(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    let mut fields = AHashMap::new();
    fields.insert("Length".to_string(), rng.gen_range(50.0..1000.0));
    fields.insert("Width".to_string(), rng.gen_range(1.0..20.0));
    fields.insert("Area".to_string(), rng.gen_range(50.0..20000.0));
    fields
}

fn generate_inner_tear_event(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    let mut fields = AHashMap::new();
    fields.insert("Length".to_string(), rng.gen_range(100.0..800.0));
    fields.insert("Width".to_string(), rng.gen_range(2.0..15.0));
    fields
}

fn generate_branch_event(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    let mut fields = AHashMap::new();
    fields.insert("Diameter".to_string(), rng.gen_range(10.0..80.0));
    fields.insert("Length".to_string(), rng.gen_range(10.0..80.0));
    fields
}

fn generate_bark_event(rng: &mut ThreadRng) -> AHashMap<String, f64> {
    let mut fields = AHashMap::new();
    fields.insert("Length".to_string(), rng.gen_range(100.0..1000.0));
    fields.insert("Width".to_string(), rng.gen_range(10.0..200.0));
    fields
}
