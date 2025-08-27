# Hantei - Recipe Compilation and Evaluation Engine

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/lumisxh/hantei/workflows/Release%20and%20Documentation/badge.svg)](https://github.com/lumisxh/hantei/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://lumisxh.github.io/hantei/)

**Hantei** is a high-performance recipe compilation and evaluation engine that transforms node-based decision trees into optimized Abstract Syntax Trees (ASTs). Built with Rust's type safety and performance in mind, Hantei compiles UI-based recipes ahead of time for lightning-fast runtime evaluation.

> **Note**: This project is actively developed. The API is stabilizing but breaking changes may occur before a 1.0 release.

## Features

- **High Performance**: Compile-time optimization with constant folding and logical simplification.
- **Extensible & Format-Agnostic**: Decoupled from the input format. Use the provided JSON parser or easily integrate your own recipe format. Add new node types without modifying the core compiler.
- **Python Bindings**: A simple, fast, and idiomatic Python API powered by PyO3.
- **Type-Safe AST**: Strongly typed expression trees with comprehensive error handling.
- **Cross-Product Evaluation**: Efficiently handles combinations of dynamic events.
- **Debug Output**: Detailed AST visualization and compilation traces via the CLI.

## Installation

### Rust Library

Add Hantei to your `Cargo.toml`:

```toml
[dependencies]
hantei = { git = "https://github.com/lumisxh/hantei", tag = "v0.2.0" }
```

### Python Bindings

**Prerequisites:**

- A recent version of the Rust toolchain (install via [rustup.rs](https://rustup.rs/))
- Python 3.11+ and `pip`

**Installation**

```bash
# Make sure your Python virtual environment is active

# Recommended: Install a specific version/tag
pip install git+https://github.com/lumisxh/hantei.git@v0.2.0

# Or, to install the latest development version from the main branch:
pip install git+https://github.com/lumisxh/hantei.git
```

This command will compile the Rust extension module and install it into your Python environment.

## Quick Example (Rust)

The new architecture decouples the compiler from the input format. The workflow involves parsing your format into Hantei's internal `FlowDefinition`.

```rust
use hantei::prelude::*;
use std::fs;

// In a real application, these structs would deserialize your specific JSON format.
// For this example, we'll assume they exist and are populated.
// The CLI tool contains a full implementation for the default JSON format.
use hantei::recipe::{FlowDefinition, Quality, IntoFlow};
use hantei::error::RecipeConversionError;

// Assume you have a struct that represents your custom recipe format.
pub struct MyRecipeFormat { /* ... fields for nodes and edges ... */ }

// You implement the `IntoFlow` trait to convert it to Hantei's model.
impl IntoFlow for MyRecipeFormat {
    fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> {
        // ... conversion logic here ...
        Ok(FlowDefinition::default()) // Placeholder
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load your custom recipe format and qualities
    let my_recipe: MyRecipeFormat = MyRecipeFormat { /* ... */ };
    let qualities: Vec<Quality> = vec![]; // Load your qualities

    // 2. Convert your format into Hantei's canonical `FlowDefinition`
    let flow_definition = my_recipe.into_flow()?;

    // 3. Use the builder to create and run the compiler
    let compiler = Compiler::builder(flow_definition, qualities).build();
    let compiled_paths = compiler.compile()?;

    // 4. Load data and evaluate
    let sample_data = SampleData::default(); // Load your actual data
    let evaluator = Evaluator::new(compiled_paths);
    let result = evaluator.eval(sample_data.static_data(), sample_data.dynamic_data())?;

    match result.quality_name {
        Some(name) => println!("Triggered: {} - {}", name, result.reason),
        None => println!("No quality triggered"),
    }

    Ok(())
}
```

## Python Usage

The Python API provides a simple "compile once, evaluate many" workflow.

```python
import hantei
import json

# 1. Load recipe and quality JSON from files
with open("data/flow.json", "r") as f:
    recipe_json = f.read()
with open("data/qualities_becker.json", "r") as f:
    qualities_json = f.read()

try:
    # 2. Initialize the Hantei class (this parses and compiles the recipe)
    evaluator = hantei.Hantei(recipe_json, qualities_json)

    # 3. Load sample data
    with open("data/sample_data.json", "r") as f:
        data = json.load(f)

    # 4. Evaluate using standard Python dictionaries
    result = evaluator.evaluate(data["static_data"], data["dynamic_data"])

    # 5. The result is a dictionary
    print(json.dumps(result, indent=2))

except (ValueError, RuntimeError) as e:
    print(f"An error occurred: {e}")
```

The `Hantei` class handles compilation in its constructor. For advanced use cases, like mapping custom node type names, a builder pattern is also available. See the [full Python API documentation](./python_api.md) for more details.

## Extensibility

Hantei is designed to be highly extensible:

- **Custom Recipe Formats**: Implement the `IntoFlow` trait on your own data structures to allow the compiler to process any recipe format, not just the default JSON.
- **Custom Node Types**: The `CompilerBuilder` allows you to register your own custom `NodeParser` implementations. This means you can define new operations (e.g., `sqrt`, `modulo`) and teach the compiler how to translate them into the AST without modifying the core engine.

## Default Input Format

While the core engine is format-agnostic, the Python bindings and CLI tool provide a built-in parser for a default JSON format.

### Recipe Flow (JSON)

```json
{
    "nodes": [
        {
            "id": "0001",
            "data": { "nodeData": { "realNodeType": "gtNode", "values": [null, 25.5] } }
        }
    ],
    "edges": [
        {
            "source": "0001",
            "target": "0002",
            "sourceHandle": "...",
            "targetHandle": "..."
        }
    ]
}
```

### Supported Node Types

| Node Type     | Operation | Description        |
| ------------- | --------- | ------------------ |
| `gtNode`      | `>`       | Greater than       |
| `stNode`      | `<`       | Smaller than       |
| `andNode`     | `AND`     | Logical AND        |
| `orNode`      | `OR`      | Logical OR         |
| `sumNode`     | `+`       | Addition           |
| `dynamicNode` | Input     | Data source        |
| ...           | ...       | _And many more..._ |

### Quality & Sample Data (JSON)

The format for qualities and sample data remains a simple and direct JSON structure. (See `data/` directory for examples).

## CLI Usage

The included CLI tool is perfect for testing, debugging, and generating AST files.

```bash
# The CLI uses the default JSON parser to run an end-to-end evaluation
cargo run --release --bin hantei-cli --features hantei-cli -- \
    data/flow.json \
    data/qualities_becker.json \
    data/sample_data.json
```

**Output Files:**

- `tmp/quality_*_naive_ast.txt` - Unoptimized AST for each quality.
- `tmp/quality_*_optimized_ast.txt` - Optimized AST for each quality.

## Library API

### Core Types

````rust
use hantei::prelude::*;

// Compilation
let flow: FlowDefinition = my_custom_format.into_flow()?;
let qualities: Vec<Quality> = load_qualities()?;
let compiler = Compiler::builder(flow, qualities).build();
let compiled_paths = compiler.compile()?;

// Evaluation
let evaluator = Evaluator::new(compiled_paths);
let result = evaluator.eval(&static_data, &dynamic_data)?;```

### Error Handling

The error types are now more descriptive, providing better context for debugging.

```rust
use hantei::error::CompileError;

// Compilation errors
match compiler.compile() {
    Err(CompileError::NodeNotFound { missing_node_id, source_node_id }) => {
        // Now you know *who* was looking for the missing node
    },
    // ... other error variants
}
````

## Documentation

Comprehensive API documentation is automatically generated and available at:
**[https://lumisxh.github.io/Hantei/](https://lumisxh.github.io/Hantei/)**

## Contributing

Contributions are welcome! Please see the API documentation for development guidelines and architecture details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
