# Hantei - Recipe Compilation and Evaluation Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/lumisxh/hantei/workflows/Release%20and%20Documentation/badge.svg)](https://github.com/lumisxh/hantei/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://lumisxh.github.io/hantei/)

**Hantei** is a high-performance recipe compilation and evaluation engine designed to transform complex, node-based decision logic into a portable, highly optimized binary format. It allows you to **compile your logic once** and **evaluate it millions of times at maximum speed**, anywhere.

At its core, Hantei uses a multi-stage compilation process that converts human-readable recipes into an optimized Abstract Syntax Tree (AST), and then "lowers" that AST into a final executable format. The primary backend is a custom-built, high-performance **register-based virtual machine**.

> **Note**: This project is actively developed. The API is stabilizing but breaking changes may occur before a 1.0 release.

## Core Workflow: Compile Once, Run Anywhere

The Hantei engine is designed for a robust production pipeline that separates slow compilation from fast evaluation.

1.  **Convert**: Use the `IntoFlow` trait to translate your custom recipe format (e.g., from a UI's JSON output) into Hantei's canonical `FlowDefinition`.
2.  **Compile to Artifacts**: The `Compiler` performs advanced AST optimizations like constant folding, dead code elimination, and common subexpression elimination, producing a set of backend-agnostic `CompilationArtifacts`.
3.  **Save (Optional)**: Use a backend to compile these artifacts into a single, serializable `CompiledRecipe` object and save it to a binary `.hanteic` file. This is your portable, pre-compiled logic.
4.  **Load & Evaluate**: In your high-performance environment, load the `.hanteic` file instantly into an `Evaluator` and run it against millions of data points with minimal overhead.

## Features

- **High Performance**: A custom-built register-based VM delivers exceptional evaluation speed, outperforming traditional tree-walking interpreters by a significant margin.
- **Serializable Compiled Artifacts**: Save the result of the entire compilation pipeline to a single binary file for fast, JIT-free startup in production.
- **Advanced AST Optimization**: Includes constant folding, algebraic simplification, dead code elimination, and common subexpression elimination.
- **Input Hashing**: Automatically converts string-based inputs (e.g., `"hole.Diameter"`) into integer IDs at compile time for lightning-fast lookups at runtime.
- **Extensible & Format-Agnostic**: Decoupled from the input format. Use the `IntoFlow` trait to support any recipe format.
- **Python Bindings**: A simple, fast, and idiomatic Python API powered by PyO3.
- **Debug Tooling**: Optional feature (`debug-tools`) provides detailed AST and bytecode visualizers for deep inspection of the compilation process.

## Quick Example (Rust)

This example demonstrates the full "compile-and-save, then load-and-run" workflow.

```rust
use hantei::prelude::*;
use hantei::backend::BackendChoice;
use hantei::recipe::{FlowDefinition, Quality, IntoFlow, CompiledRecipe};
use hantei::error::RecipeConversionError;
use ahash::AHashMap;

// Assume MyRecipe and its IntoFlow impl exist...
# struct MyRecipe;
# impl IntoFlow for MyRecipe { fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> { Ok(FlowDefinition::default()) } }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- COMPILE-TIME (e.g., in a build script or CLI tool) ---

    let my_recipe = MyRecipe { /* ... */ };
    let qualities = vec![Quality { name: "Premium".to_string(), priority: 1 }];
    let flow_definition = my_recipe.into_flow()?;

    // 1. Compile the flow into optimized, backend-agnostic artifacts.
    let compiler = Compiler::builder(flow_definition, qualities).build();
    let artifacts = compiler.compile()?;

    // 2. "Lower" the artifacts into a serializable format using a chosen backend.
    let bytecode_backend = hantei::bytecode::BytecodeBackend;
    let compiled_recipe_for_saving = bytecode_backend.compile(artifacts)?;

    // 3. Save the compiled recipe to a file.
    let artifact_path = "premium_recipe.hanteic";
    compiled_recipe_for_saving.save(artifact_path)?;
    println!("Recipe compiled and saved to {}", artifact_path);


    // --- RUNTIME (e.g., on a production server) ---

    // 4. Load the pre-compiled recipe directly into an evaluator.
    // This step is extremely fast as no AST optimization or code generation happens here.
    let evaluator = Evaluator::from_file(BackendChoice::Bytecode, artifact_path)?;

    let static_data = AHashMap::new(); // Populate with real data
    let dynamic_data = AHashMap::new();

    // 5. Evaluate data at high speed.
    let result = evaluator.eval(&static_data, &dynamic_data)?;
    println!("Evaluation result: {:?}", result.quality_name);

    Ok(())
}
```

## Python Usage

The Python API hides the complexity of compilation, providing a simple and fast interface.

```python
import hantei
import json

# 1. Load recipe and quality JSON from files
with open("data/flow.json", "r") as f:
    recipe_json = f.read()
with open("data/qualities.json", "r") as f:
    qualities_json = f.read()

try:
    # 2. Initialize the Hantei class. This parses and compiles the recipe in one step.
    #    The 'bytecode' backend is used by default for maximum performance.
    evaluator = hantei.Hantei(recipe_json, qualities_json)

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

## CLI Usage

The CLI is the primary tool for compiling recipes and running benchmarks. It now supports a compile/run workflow with binary artifacts.

#### Compile a recipe to a binary artifact:

```bash
# Use the bytecode backend for the best performance
cargo run --release --bin hantei-cli --features "hantei-cli" -- \
    compile data/flow.json data/qualities.json \
    --backend bytecode \
    -o my_recipe.hanteic
```

#### Run a pre-compiled artifact against some data:

```bash
cargo run --release --bin hantei-cli --features "hantei-cli" -- \
    run my_recipe.hanteic data/sample_data.json \
    --backend bytecode
```

#### Run a full benchmark comparing both backends:

```bash
# Use the debug-tools feature to see detailed output
cargo run --release --bin hantei-cli --features "hantei-cli,debug-tools" -- \
    --benchmark 100 \
    data/flow.json \
    data/qualities.json \
    data/sample_data.json
```

## Performance

The engine offers two backends with different performance profiles:

- **Bytecode (Default & Recommended)**: Offers the highest evaluation performance due to its register-based VM, input hashing, and short-circuiting logic. It has a slightly higher one-time compilation cost but is significantly faster for runtime evaluation.
- **Interpreter**: A direct tree-walking interpreter. It is slightly faster to compile but slower to evaluate. Its primary advantage is providing extremely detailed, step-by-step evaluation traces, making it an invaluable tool for debugging complex logic.

## Documentation

Comprehensive API documentation is automatically generated and available at:
**[https://lumisxh.github.io/Hantei/](https://lumisxh.github.io/Hantei/)**
