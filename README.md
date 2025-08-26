# Hantei - Recipe Compilation and Evaluation Engine

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/lumisxh/hantei/workflows/Release%20and%20Documentation/badge.svg)](https://github.com/lumisxh/hantei/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://lumisxh.github.io/hantei/)

**Hantei** is a high-performance recipe compilation and evaluation engine that transforms node-based decision trees into optimized Abstract Syntax Trees (ASTs). Built with Rust's type safety and performance in mind, Hantei compiles UI-based recipes ahead of time for lightning-fast runtime evaluation.

> **Note**: This project is currently in development and not yet ready for production use.

## Features

- **High Performance**: Compile-time optimization with constant folding and logical simplification
- **Type-Safe AST**: Strongly typed expression trees with comprehensive error handling
- **Modular Architecture**: Clean separation between compilation, evaluation, and data handling
- **Cross-Product Evaluation**: Efficient handling of dynamic event combinations
- **Debug Output**: Detailed AST visualization and compilation traces
- **Zero-Runtime Overhead**: All recipe logic compiled ahead of time
- **Extensible Design**: Easy to add new node types and operations
- **Memory Efficient**: Optimized data structures with minimal allocations

## Installation

Add Hantei to your `Cargo.toml`:

```toml
[dependencies]
hantei = { git = "https://github.com/lumisxh/hantei", tag = "v0.1.0" }

# For library use only (without CLI)
hantei = { git = "https://github.com/lumisxh/hantei", tag = "v0.1.0", default-features = false }
```

## Quick Example

```rust
use hantei::{Compiler, Evaluator, SampleData};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load recipe and quality definitions
    let recipe_json = std::fs::read_to_string("recipe.json")?;
    let qualities_json = std::fs::read_to_string("qualities.json")?;

    // Compile recipe into optimized ASTs
    let compiler = Compiler::new(&recipe_json, &qualities_json)?;
    let (_logical_repr, compiled_paths) = compiler.compile()?;

    println!("Compiled {} quality paths", compiled_paths.len());

    // Load sample data
    let sample_data = SampleData::from_file("sample_data.json")?;

    // Evaluate against sample data
    let evaluator = Evaluator::new(compiled_paths);
    let result = evaluator.eval(sample_data.static_data(), sample_data.dynamic_data())?;

    match result.quality_name {
        Some(name) => println!("Triggered: {} - {}", name, result.reason),
        None => println!("No quality triggered"),
    }

    Ok(())
}
```

## Input Format Specifications

### Recipe Flow (JSON)

The recipe flow defines the node-based decision tree structure:

```json
{
    "nodes": [
        {
            "id": "0001",
            "data": {
                "nodeData": {
                    "name": "Greater than",
                    "realNodeType": "gtNode",
                    "values": [null, 25.5],
                    "handles": [
                        /* handle definitions */
                    ]
                }
            }
        }
    ],
    "edges": [
        {
            "source": "0001",
            "target": "0002",
            "sourceHandle": "bool-bool-0001-2",
            "targetHandle": "bool-bool-0002-0"
        }
    ]
}
```

#### Supported Node Types

| Node Type     | Operation | Inputs     | Output  | Description             |
| ------------- | --------- | ---------- | ------- | ----------------------- |
| `gtNode`      | `>`       | 2 numbers  | boolean | Greater than comparison |
| `stNode`      | `<`       | 2 numbers  | boolean | Smaller than comparison |
| `gteqNode`    | `>=`      | 2 numbers  | boolean | Greater than or equal   |
| `steqNode`    | `<=`      | 2 numbers  | boolean | Smaller than or equal   |
| `eqNode`      | `==`      | 2 values   | boolean | Equality comparison     |
| `andNode`     | `AND`     | 2 booleans | boolean | Logical AND             |
| `orNode`      | `OR`      | 2 booleans | boolean | Logical OR              |
| `notNode`     | `NOT`     | 1 boolean  | boolean | Logical negation        |
| `sumNode`     | `+`       | 2 numbers  | number  | Addition                |
| `subNode`     | `-`       | 2 numbers  | number  | Subtraction             |
| `multNode`    | `*`       | 2 numbers  | number  | Multiplication          |
| `divideNode`  | `/`       | 2 numbers  | number  | Division                |
| `dynamicNode` | Input     | -          | varies  | Data input node         |

#### Dynamic Nodes (Data Sources)

Dynamic nodes represent data inputs and can be either static or event-based:

```json
{
    "id": "0010",
    "data": {
        "nodeData": {
            "name": "Hole",
            "realNodeType": "dynamicNode",
            "realInputType": "hole", // null for static data
            "cases": [
                {
                    "caseId": 0,
                    "caseName": "Diameter",
                    "realCaseType": "number"
                },
                {
                    "caseId": 1,
                    "caseName": "Length",
                    "realCaseType": "number"
                }
            ]
        }
    }
}
```

- **Static nodes**: `realInputType` is `null`, data comes from static_data
- **Event nodes**: `realInputType` specifies event type, data comes from dynamic_data

### Quality Definitions (JSON)

Quality definitions specify the possible outcomes and their evaluation priorities:

```json
[
    { "id": 0, "name": "Premium", "priority": 1, "negated": false },
    { "id": 1, "name": "Standard", "priority": 2, "negated": false },
    { "id": 2, "name": "Defective", "priority": 3, "negated": false }
]
```

**Fields:**

- `id`: Unique identifier
- `name`: Human-readable quality name
- `priority`: Evaluation order (1 = highest priority)
- `negated`: Whether to invert the result (currently unused)

### Sample Data (JSON)

Sample data provides the runtime values for evaluation:

```json
{
    "static_data": {
        "Temperature": 25.5,
        "Humidity": 60.0,
        "Pressure": 1013.25
    },
    "dynamic_data": {
        "hole": [
            { "Diameter": 5.2, "Length": 12.0 },
            { "Diameter": 8.7, "Length": 15.5 }
        ],
        "tear": [{ "Length": 25.0, "Width": 2.1 }]
    }
}
```

**Structure:**

- `static_data`: Key-value pairs for static measurements
- `dynamic_data`: Event types with arrays of detected instances

## CLI Usage

The included CLI tool provides easy testing and evaluation:

```bash
# Basic usage
cargo run -- recipe.json qualities.json sample_data.json

# With default mock data
cargo run -- recipe.json qualities.json

# Check compilation without evaluation
cargo check
```

**Output Files:**

- `tmp/logical_connections.txt` - Connection graph visualization
- `tmp/quality_*_naive_ast.txt` - Unoptimized AST for each quality
- `tmp/quality_*_optimized_ast.txt` - Optimized AST for each quality

## Library API

### Core Types

```rust
use hantei::{Compiler, Evaluator, SampleData, Expression, Value};

// Compilation
let compiler = Compiler::new(&recipe_json, &qualities_json)?;
let (logical_repr, compiled_paths) = compiler.compile()?;

// Evaluation
let evaluator = Evaluator::new(compiled_paths);
let result = evaluator.eval(static_data, dynamic_data)?;

// Data loading
let data = SampleData::from_file("data.json")?;
let default_data = SampleData::default();
```

### AST Expressions

The `Expression` enum represents all possible AST nodes:

```rust
pub enum Expression {
    // Arithmetic
    Sum(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),

    // Comparison
    GreaterThan(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),

    // Logic
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),

    // Leaf nodes
    Literal(Value),
    Input(InputSource),
}
```

### Error Handling

```rust
use hantei::{CompileError, EvaluationError};

// Compilation errors
match compiler.compile() {
    Err(CompileError::NodeNotFound(id)) => { /* handle */ },
    Err(CompileError::JsonParseError(msg)) => { /* handle */ },
    Ok(result) => { /* success */ },
}

// Evaluation errors
match evaluator.eval(static_data, dynamic_data) {
    Err(EvaluationError::InputNotFound(name)) => { /* handle */ },
    Err(EvaluationError::TypeMismatch { expected, found }) => { /* handle */ },
    Ok(result) => { /* success */ },
}
```

## Documentation

### API Documentation

Comprehensive API documentation is automatically generated and available at:
**[https://lumisxh.github.io/Hantei/](https://lumisxh.github.io/Hantei/)**

The documentation includes:

- Complete API reference with examples
- Recipe compilation process and optimization
- AST structure and evaluation engine
- Input format specifications and validation
- Error handling patterns and best practices
- Performance characteristics and benchmarks

## Development Status

This library is actively developed with automated testing and security auditing. Check the [Actions page](https://github.com/lumisxh/hantei/actions) for current build status and release information.

## Contributing

Contributions are welcome! Please see the API documentation for development guidelines, architecture details, and instructions for implementing new node types and operations.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
