# Hantei Command-Line Tools

This directory contains optional, feature-gated command-line tools and utilities for the Hantei evaluation engine. These tools are intended for debugging, testing, and data generation.

## Tools Overview

1.  **Hantei CLI (`hantei-cli`)**: The main debugging and evaluation runner.
2.  **Data Generator (`data-gen`)**: A utility for creating randomized test data.
3.  **Recipe Visualizer (`visualizer.html`)**: An interactive, browser-based tool to view recipe graphs.

---

### 1. Hantei CLI

**Source**: `tools/hantei-cli/main.rs`
**Feature Flag**: `hantei-cli`

This tool provides a command-line interface to the full compilation and evaluation pipeline. It is the primary way to test a recipe from end to end, providing detailed performance metrics and optional debug artifacts.

#### Building and Running

To build and run the CLI, you must enable the `hantei-cli` feature:

```bash
# Basic syntax
cargo run --features hantei-cli -- [OPTIONS] <RECIPE> <QUALITIES> [DATA]

# Example run with sample data
cargo run --features hantei-cli -- \
    data/flow.json \
    data/qualities_becker.json \
    data/sample_data.json
```

#### Arguments

- `recipe_path`: (Required) Path to the recipe flow JSON file.
- `qualities_path`: (Required) Path to the qualities definition JSON file.
- `sample_data_path`: (Optional) Path to a sample data JSON file. If omitted, default mock data is used.

#### Options

- `--help`: Display the help message with all arguments and options.

---

### 2. Data Generator

**Source**: `tools/data-generator/main.rs`
**Feature Flag**: `data-gen`

This tool generates a `sample_data.json` file with randomized values. It is highly useful for creating a wide variety of test cases to validate the evaluation logic. The generation parameters (e.g., value ranges, number of events) are configured directly in the `main.rs` file.

#### Building and Running

To build and run the data generator, enable the `data-gen` feature and specify the binary name:

```bash
# Generate a file named `generated_data.json` in the project root
cargo run --bin data-gen --features data-gen

# Generate a file with a custom name
cargo run --bin data-gen --features data-gen -- --output my_custom_data.json
```

---

### 3. Recipe Visualizer

**Source**: `tools/visualizer.html`

This is a self-contained HTML file for graphically rendering a `flow.json` recipe. It is incredibly useful for visually inspecting and debugging a recipe's structure, verifying that node connections and data flow are correct before analyzing the AST output.

It requires no compilation, server, or feature flags to run.

#### How to Use

1.  Open the `tools/visualizer.html` file directly in a modern web browser (e.g., Chrome, Firefox, Edge).
2.  Click the **"Choose File"** button.
3.  Select your `flow.json` file to render the graph.

#### Features

- **Interactive View**: Pan and zoom the graph to inspect complex recipes.
- **Dark Mode**: Includes a "Toggle Theme" button for better visibility.
- **Clear Data Flow**: Nodes are rendered with inputs on the left and outputs on the right for a clear, logical layout.
- **Detailed Nodes**: Nodes display their ID, type, and any literal values they contain.
