# Hantei - Python API Documentation

This document provides a comprehensive overview of the Hantei Python API, which offers high-performance recipe evaluation by wrapping the core Rust engine. These bindings are designed to feel idiomatic and integrate seamlessly into Python workflows, allowing you to leverage Rust's speed without leaving the comfort of Python.

## Core Concepts

The Python API simplifies the Hantei engine into a powerful, configurable workflow:

1.  **Initialize Once**: Create an instance of the `Hantei` class. For simple cases, you can do this directly. for advanced customization, you use the `Hantei.builder()`. This step always compiles the recipe into an optimized, in-memory representation.
2.  **Evaluate Many Times**: Call the `.evaluate()` method on the instance with your `static_data` and `dynamic_data` dictionaries. This evaluation step is extremely fast, as the complex logic has already been compiled.

## The `hantei.Hantei` Class

This is the main entry point for using the Hantei engine in Python. It handles the compilation of the recipe and provides a method for evaluation.

---

### Simple Initialization

For most use cases, you can initialize the evaluator directly from your JSON strings.

#### `Hantei(recipe_json, qualities_json)`

The standard constructor for the `Hantei` class. It loads, parses, and compiles the provided recipe and quality definitions with default settings.

- **Parameters:**
    - `recipe_json` (str): A string containing the entire JSON content of the recipe flow (e.g., from `flow.json`).
    - `qualities_json` (str): A string containing the JSON array of quality definitions (e.g., from `qualities_becker.json`).
- **Returns:**
    - An instance of the `Hantei` class, ready for evaluation.
- **Raises:**
    - `ValueError`: If the JSON is malformed, a required node is missing, or any other compilation error occurs.

---

### Advanced Usage: The Builder Pattern

If you need to customize the compilation process—for example, by using different names for node types—you should use the builder pattern.

#### `Hantei.builder(recipe_json, qualities_json)`

A class method that returns a `HanteiBuilder` instance, allowing you to configure the compiler before building the final evaluator.

- **Parameters:**
    - `recipe_json` (str): The recipe flow JSON string.
    - `qualities_json` (str): The qualities definition JSON string.
- **Returns:**
    - A `HanteiBuilder` instance.

#### `HanteiBuilder.with_type_mapping(custom_name, hantei_name)`

Configures a custom name mapping. This is useful if your recipe JSON uses names like `"CompareGreaterThan"` instead of Hantei's internal `"gtNode"`.

- **Parameters:**
    - `custom_name` (str): The node type name used in your JSON file.
    - `hantei_name` (str): The corresponding internal Hantei node type name (e.g., `"gtNode"`, `"andNode"`).
- **Returns:**
    - The `HanteiBuilder` instance, to allow for chaining calls.

#### `HanteiBuilder.build()`

Consumes the builder and constructs the final, compiled `Hantei` instance.

- **Returns:**
    - An instance of the `Hantei` class.
- **Raises:**
    - `ValueError`: If any compilation errors occur during the build.

**Example of the Builder Pattern:**

```python
# Assume my_recipe.json uses "GreaterThan" instead of "gtNode"
builder = hantei.Hantei.builder(my_recipe_json, qualities_json)

# Map the custom name to the internal Hantei name
builder.with_type_mapping("GreaterThan", "gtNode")

# Build the final evaluator
evaluator = builder.build()
```

---

### `evaluate(static_data, dynamic_data)`

Evaluates the pre-compiled recipe against a set of runtime data. This method is the same whether you used the direct constructor or the builder.

- **Parameters:**
    - `static_data` (dict): A dictionary containing the static measurements.
        - **Keys** are `str` representing the measurement name (e.g., `"Temperature"`).
        - **Values** are `float` or `int`.
    - `dynamic_data` (dict): A dictionary containing the dynamic, event-based data.
        - **Keys** are `str` representing the event type (e.g., `"hole"`).
        - **Values** are a `list` of dictionaries, where each inner dictionary represents a single detected event instance.
- **Returns:**
    - A `dict` containing the evaluation result with the following keys:
        - `quality_name` (str | None): The name of the highest-priority quality that was triggered. `None` if no quality was triggered.
        - `quality_priority` (int | None): The priority of the triggered quality. `None` if no quality was triggered.
        - `reason` (str): A human-readable explanation of the evaluation path that led to the result.
- **Raises:**
    - `RuntimeError`: If an evaluation error occurs, such as a type mismatch in the AST or a required input value not being found in the provided data.

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

        with open("data/qualities_becker.json", "r") as f:
            qualities_json = f.read()

        # 2. Initialize the Hantei evaluator. This compiles the recipe.
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

        # 5. Print the results
        print("\n--- Evaluation Result ---")
        if result["quality_name"]:
            print(f"  Triggered Quality: {result['quality_name']} (Priority: {result['quality_priority']})")
            print(f"  Reason: {result['reason']}")
        else:
            print("  No quality was triggered.")

    except (ValueError, RuntimeError) as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    run_evaluation()
```
