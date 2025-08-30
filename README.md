# Hantei - Recipe Compilation and Evaluation Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/lumisxh/hantei/workflows/Release%20and%20Documentation/badge.svg)](https://github.com/lumisxh/hantei/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://lumisxh.github.io/hantei/)

**Hantei** is a high-performance recipe compilation and evaluation engine that transforms node-based decision trees into optimized Abstract Syntax Trees (ASTs). Built with Rust's type safety and performance in mind, Hantei compiles UI-based recipes ahead of time for lightning-fast runtime evaluation using selectable backends.

> **Note**: This project is actively developed. The API is stabilizing but breaking changes may occur before a 1.0 release.

## Features

- **High Performance**: Compile-time optimization with constant folding and logical simplification.
- **Selectable Backends**: Choose between a direct AST **Interpreter** for easy debugging or a **Bytecode VM** for faster runtime execution.
- **Extensible & Format-Agnostic**: Decoupled from the input format. Use the provided JSON parser or easily integrate your own recipe format. Add new node types without modifying the core compiler.
- **Python Bindings**: A simple, fast, and idiomatic Python API powered by PyO3.
- **Type-Safe Architecture**: Strongly typed expression trees and backends with comprehensive, distinct error handling for each stage of the process.
- **Cross-Product Evaluation**: Efficiently handles combinations of dynamic events across both backends.
- **Debug Output**: Detailed AST and bytecode visualization via the CLI.

## Installation

### Rust Library

Add Hantei to your `Cargo.toml`:

```toml
[dependencies]
hantei = { git = "https://github.com/lumisxh/hantei", tag = "v0.3.0" }
```

### Python Bindings

**Prerequisites:**

- A recent version of the Rust toolchain (install via [rustup.rs](https://rustup.rs/))
- Python 3.11+ and `pip`

**Installation**

```bash
# Make sure your Python virtual environment is active

# Recommended: Install a fixed version from GitHub:
pip install git+https://github.com/lumisxh/hantei.git@v0.3.0

# Or, to install the latest development version from the main branch:
pip install git+https://github.com/lumisxh/hantei.git
```

This command will compile the Rust extension module and install it into your Python environment.

## Quick Example (Rust)

The new architecture decouples the compiler from the input format and allows for backend selection.

```rust
use hantei::prelude::*;
use hantei::backend::BackendChoice; // Import the backend choice
use std::fs;

// In a real application, these structs would deserialize your specific JSON format.
// The CLI tool contains a full implementation for the default JSON format.
use hantei::recipe::{FlowDefinition, Quality, IntoFlow};
use hantei::error::RecipeConversionError;

pub struct MyRecipeFormat { /* ... fields for nodes and edges ... */ }

impl IntoFlow for MyRecipeFormat {
    fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> {
        // ... your conversion logic here ...
        Ok(FlowDefinition::default()) // Placeholder
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load and convert your custom recipe format
    let my_recipe: MyRecipeFormat = MyRecipeFormat { /* ... */ };
    let qualities: Vec<Quality> = vec![]; // Load your qualities
    let flow_definition = my_recipe.into_flow()?;

    // 2. Build the ASTs from the flow definition
    let compiler = Compiler::builder(flow_definition, qualities).build();
    let compiled_paths = compiler.compile()?;

    // 3. Create an evaluator by choosing a backend and providing the ASTs
    //    This step performs backend-specific compilation (e.g., to bytecode).
    let evaluator = Evaluator::new(BackendChoice::Bytecode, compiled_paths)?;

    // 4. Load data and evaluate
    let sample_data = SampleData::default(); // Load your actual data
    let result = evaluator.eval(sample_data.static_data(), sample_data.dynamic_data())?;

    match result.quality_name {
        Some(name) => println!("Triggered: {} - {}", name, result.reason),
        None => println!("No quality triggered"),
    }

    Ok(())
}
```

## Python Usage

The Python API provides a simple "compile once, evaluate many" workflow and allows for backend selection.

```python
import hantei
import json

# 1. Load recipe and quality JSON from files
with open("data/flow.json", "r") as f:
    recipe_json = f.read()
with open("data/qualities.json", "r") as f:
    qualities_json = f.read()

try:
    # 2. Initialize the Hantei class. This parses and compiles the recipe.
    #    The 'bytecode' backend is used by default for performance.
    evaluator = hantei.Hantei(recipe_json, qualities_json)

    # Or, choose the interpreter explicitly for better debugging traces
    # evaluator = hantei.Hantei(recipe_json, qualities_json, backend="interpreter")

    # 3. Load sample data
    with open("data/sample_data.json", "r") as f:
        data = json.load(f)

    # 4. Evaluate using standard Python dictionaries
    result = evaluator.evaluate(data["static_data"], data["dynamic_data"])

    # 5. The result is a dedicated result class with properties
    print(f"Quality: {result.quality_name}")
    print(f"Priority: {result.quality_priority}")
    print(f"Reason: {result.reason}")

except (ValueError, RuntimeError) as e:
    print(f"An error occurred: {e}")
```

See the [full Python API documentation](./python_api.md) for more details.

## CLI Usage

The CLI tool is perfect for testing and debugging. It now supports a `--backend` flag.

```bash
# Run with the faster bytecode backend
cargo run --release --bin hantei-cli --features hantei-cli -- \
    data/flow.json \
    data/qualities.json \
    data/sample_data.json \
    --backend bytecode

# Run with the interpreter for more detailed trace reasons
cargo run --release --bin hantei-cli --features hantei-cli -- \
    data/flow.json \
    data/qualities.json \
    data/sample_data.json \
    --backend interpreter
```

## Library API

### Core Types

```rust
use hantei::prelude::*;
use hantei::backend::BackendChoice;

// Frontend Compilation (Recipe -> AST)
let flow: FlowDefinition = my_custom_format.into_flow()?;
let qualities: Vec<Quality> = load_qualities()?;
let compiler = Compiler::builder(flow, qualities).build();
let compiled_paths = compiler.compile()?;

// Backend Compilation & Evaluation
let evaluator = Evaluator::new(BackendChoice::Bytecode, compiled_paths)?;
let result = evaluator.eval(&static_data, &dynamic_data)?;
```

### Error Handling

The error types are now distinct for each stage of the process, providing clear context.

```rust
use hantei::error::{AstBuildError, BackendError};

// Frontend compilation errors
match compiler.compile() {
    Err(AstBuildError::NodeNotFound { .. }) => { /* ... */ },
    _ => {}
}

// Backend compilation errors
match Evaluator::new(BackendChoice::Bytecode, compiled_paths) {
    Err(BackendError::UnsupportedAstNode(msg)) => {
        // The bytecode backend doesn't support a specific AST node
    },
    _ => {}
}
```

## Documentation

Comprehensive API documentation is automatically generated and available at:
**[https://lumisxh.github.io/Hantei/](https://lumisxh.github.io/Hantei/)**
