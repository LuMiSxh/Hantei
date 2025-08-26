# Hantei - A Recipe Evaluator POC

This project is a Proof-of-Concept (POC) for a high-performance, node-based recipe evaluator written in Rust. Its primary purpose is to explore an alternative to an existing Python-based system, with a focus on maximizing performance and ensuring logical correctness through a compilation-based approach.

## Core Logic and Architecture

The fundamental principle of Hantei is to treat recipes not as a graph to be interpreted at runtime, but as source code to be compiled and optimized ahead of time. The evaluation process happens in a single, one-time compilation step upon initialization.

The compilation pipeline consists of three main stages:

1.  **Parsing**: The system first parses a verbose, UI-centric JSON format that represents the recipe as a graph of nodes and edges. This format is rich with data not relevant to pure execution logic.

2.  **AST Compilation**: The parsed graph is then compiled into a set of highly efficient, logic-focused data structures called Abstract Syntax Trees (ASTs). Each path in the recipe that leads to a quality decision becomes a distinct AST. This tree represents the pure computational logic of the path, free from any UI or traversal overhead.

3.  **Optimization**: Before the ASTs are finalized, a series of optimization passes are run on them. These passes simplify the logic in the same way a programming language compiler would. Key optimizations include:
    - **Constant Folding**: Pre-calculating branches of the tree that consist only of literal values (e.g., `5 + 10` becomes `15`).
    - **Logical Simplification**: Reducing boolean logic using algebraic rules (e.g., `X AND true` becomes just `X`; `Y OR false` becomes `Y`).

The final output of this compilation is a set of minimal, hyper-optimized ASTs that are ready for immediate execution.

## How to Run the POC

This project is a command-line application that takes two required arguments and one optional argument.

**1. Prepare the Configuration Files:**

- `recipe.json`: The recipe graph exported from the UI system.
- `qualities.json`: A JSON array defining the possible quality outcomes and their priorities.
- `sample_data.json` (Optional): A JSON file containing test data for an evaluation run.

**2. Run from the Terminal:**

```bash
# Run with default mock data
cargo run -- recipe.json qualities.json

# Run with a specific data file
cargo run -- recipe.json qualities.json sample_data.json
```

The program will output a detailed log of the compilation process, including visualizations of the naive (pre-optimization) and optimized ASTs, followed by the final evaluation result.
