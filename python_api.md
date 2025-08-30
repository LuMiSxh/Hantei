# Hantei - Python API Documentation

This document provides a comprehensive overview of the Hantei Python API, which offers high-performance recipe evaluation by wrapping the core Rust engine. These bindings are designed to feel idiomatic and integrate seamlessly into Python workflows.

## Core Concepts

The Python API simplifies the Hantei engine into a powerful, configurable workflow:

1.  **Initialize Once**: Create an instance of the `hantei.Hantei` class. This step compiles the recipe into an optimized, in-memory representation using a chosen backend.
2.  **Evaluate Many Times**: Call the `.evaluate()` method on the instance with your `static_data` and `dynamic_data` dictionaries. This evaluation step is extremely fast, as the complex logic has already been compiled.

## The `hantei.Hantei` Class

This is the main entry point for using the Hantei engine in Python. It handles the compilation of the recipe and provides a method for evaluation.

### `Hantei(recipe_json, qualities_json, backend="bytecode")`

The constructor for the `Hantei` class. It loads, parses, and compiles the provided recipe and quality definitions.

- **Parameters:**
    - `recipe_json` (str): A string containing the entire JSON content of the recipe flow (e.g., from `flow.json`).
    - `qualities_json` (str): A string containing the JSON array of quality definitions (e.g., from `qualities.json`).
    - `backend` (str, optional): The execution backend to use. Defaults to `"bytecode"`.
        - `"bytecode"`: (Default) Compiles the recipe to a custom bytecode format and runs it on a fast virtual machine. Recommended for production use.
        - `"interpreter"`: Directly interprets the Abstract Syntax Tree. Slower, but can produce more detailed debugging traces in its `reason` string.
- **Returns:**
    - An instance of the `Hantei` class, ready for evaluation.
- **Raises:**
    - `ValueError`: If the JSON is malformed, a required node is missing, the backend choice is invalid, or any other compilation error occurs.

### `evaluate(static_data, dynamic_data)`

Evaluates the pre-compiled recipe against a set of runtime data.

- **Parameters:**
    - `static_data` (dict): A dictionary containing the static measurements.
        - **Keys** are `str` representing the measurement name (e.g., `"Temperature"`).
        - **Values** are `float` or `int`.
    - `dynamic_data` (dict): A dictionary containing the dynamic, event-based data.
        - **Keys** are `str` representing the event type (e.g., `"hole"`).
        - **Values** are a `list` of dictionaries, where each inner dictionary represents a single detected event instance.
- **Returns:**
    - An instance of the `hantei.EvaluationResult` class, which has the following properties:
        - `quality_name` (str | None): The name of the highest-priority quality that was triggered.
        - `quality_priority` (int | None): The priority of the triggered quality.
        - `reason` (str): A human-readable explanation of the evaluation path.
- **Raises:**
    - `RuntimeError`: If an evaluation error occurs, such as a type mismatch in the logic or a required input value not being found in the provided data.

## Data Structures

The `evaluate` method expects standard Python dictionaries with a specific structure.

#### `static_data` Format

```python
static_data = {
    "Leading width": 1970.0,
    "Trailing width": 1965.0,
    "Area": 4147000.0,
    "Humidity": 7.5
}
```

#### `dynamic_data` Format

```python
dynamic_data = {
    "hole": [
        {"Diameter": 22.0, "Area": 140.0, "Length": 15.0},
        {"Diameter": 150.0, "Area": 1950.0, "Length": 50.0}
    ],
    "tear": [
        {"Length": 100.0, "Width": 3.0, "Area": 300.0}
    ],
    "black_branch": [] # It is valid to have an event type with no instances
}
```

## Error Handling

Errors from the Rust core are propagated as Python exceptions. It is best practice to wrap calls to the Hantei API in a `try...except` block.

```python
import hantei

try:
    # Initialization might fail if JSON is invalid
    evaluator = hantei.Hantei(recipe_str, qualities_str)

    # Evaluation might fail if data is missing or causes a type error
    result = evaluator.evaluate(static_data, dynamic_data)
    print(result)

except ValueError as e:
    print(f"Compilation Error: {e}")
except RuntimeError as e:
    print(f"Evaluation Error: {e}")
```

## Complete Example

This example demonstrates the full end-to-end workflow: loading files, initializing the evaluator, and evaluating data.

```python
import hantei
import json

def run_evaluation():
    try:
        # 1. Load recipe and quality definitions from files
        with open("data/flow.json", "r") as f:
            recipe_json = f.read()

        with open("data/qualities.json", "r") as f:
            qualities_json = f.read()

        # 2. Initialize the Hantei evaluator.
        #    This compiles the recipe using the default 'bytecode' backend.
        print("Compiling recipe...")
        evaluator = hantei.Hantei(recipe_json, qualities_json)
        print("Compilation successful!")

        # 3. Load sample data
        with open("data/sample_data.json", "r") as f:
            sample_data = json.load(f)

        # 4. Evaluate the data
        print("Evaluating data...")
        result = evaluator.evaluate(
            sample_data["static_data"],
            sample_data["dynamic_data"]
        )

        # 5. Print the results from the result object's properties
        print("\n--- Evaluation Result ---")
        if result.quality_name:
            print(f"  Triggered Quality: {result.quality_name} (Priority: {result.quality_priority})")
            print(f"  Reason: {result.reason}")
        else:
            print("  No quality was triggered.")

    except (ValueError, RuntimeError) as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    run_evaluation()
```
