# Hantei Optimization Roadmap

This document outlines the next strategic phases for enhancing the performance of the Hantei engine. The optimizations are designed to build upon the existing framework, targeting specific areas of the compilation and evaluation pipeline to yield significant performance gains.

The phases are structured to move from broader, frontend optimizations that benefit both backends to more specialized, backend-specific enhancements.

---

## Phase 2: Advanced AST & Logic Optimization

**Goal**: To further reduce the complexity of the Abstract Syntax Tree before it is passed to the backends. A simpler, more efficient AST means less work for both the Interpreter and the Bytecode VM.

### Techniques to Implement:

1.  **Algebraic Simplification & Identity Transformation**:
    - **Description**: This pass will identify and simplify mathematical and logical identities. For example, an expression like `($Input.Value - $Input.Value)` can be folded into a `Literal(0)`, and `($Input.Value AND TRUE)` can be simplified to just `$Input.Value`. This goes beyond simple constant folding by working with symbolic inputs.
    - **Expected Impact**: Reduces the number of nodes in the AST, leading to faster backend compilation and fewer runtime operations.

2.  **Enhanced Dead Code Elimination (DCE)**:
    - **Description**: Generalize the existing DCE logic. The current implementation has a specific rule for `($x > A) AND ($x < B)`. This can be expanded to a more robust system that understands logical impossibilities across a wider range of conditions, such as contradictory `AND` clauses involving equality, e.g., `($x == 5) AND ($x == 10)`.
    - **Expected Impact**: Prunes entire branches from the evaluation tree, providing significant speedups by avoiding unnecessary computations, especially in the `DynamicEvaluator`.

3.  **Boolean Logic Simplification (De Morgan's Laws)**:
    - **Description**: Apply boolean algebra rules, such as De Morgan's laws, to restructure logical expressions. For example, transforming `NOT (A OR B)` into `(NOT A) AND (NOT B)`.
    - **Expected Impact**: This can "flatten" nested logical structures, exposing more opportunities for constant folding and Common Subexpression Elimination (CSE) to take effect, leading to a more optimal tree.

---

## Phase 3: Bytecode-Level Optimization

**Goal**: To optimize the low-level bytecode generated for the VM. These changes are specific to the Bytecode backend and aim to reduce the number of instructions executed and the overhead of the VM's main execution loop.

### Techniques to Implement:

1.  **Peephole Optimization**:
    - **Description**: After initial bytecode generation, a second pass will scan the instruction sequence through a small "peephole" (a sliding window of 2-3 instructions). It will replace known inefficient patterns with more efficient ones.
    - **Examples**:
        - **Redundant Load/Pop**: A sequence like `LoadStatic("X")`, `Pop` can be completely removed.
        - **Jump Chaining**: A `Jump` to another `Jump` instruction can be replaced with a single `Jump` to the final destination.
    - **Expected Impact**: Reduces the total number of instructions, directly speeding up the VM's execution loop.

2.  **Instruction Fusion**:
    - **Description**: Introduce new, more complex opcodes that combine the functionality of several existing ones. The compiler would then be taught to recognize patterns that can be "fused" into these new opcodes.
    - **Example**: The common pattern of `LoadStatic`, `Push(Constant)`, `GreaterThan` could be fused into a single new instruction: `CmpStaticGtImm("Temp", 25.0)`.
    - **Expected Impact**: Reduces the overhead of instruction fetching and dispatching within the VM, leading to a significant performance boost for common comparison patterns.

---

## Phase 4: Advanced Runtime & VM Enhancements

**Goal**: To introduce more intelligent runtime behaviors that reduce redundant work within a single evaluation, especially for recipes with complex dynamic data or repeated pure calculations.

### Techniques to Implement:

1.  **VM Result Caching for Pure Subroutines**:
    - **Description**: Enhance the VM to include a short-lived cache that exists only for the duration of a single `.eval()` call. When the VM is about to execute a `Call` to a subroutine, it first determines if the subroutine is "pure" (i.e., it only depends on static data). If it is, the VM will cache its return value. Subsequent calls to the same pure subroutine within the same evaluation will return the cached value instantly instead of re-executing the bytecode.
    - **Expected Impact**: Drastically speeds up recipes where the same static calculations are referenced in multiple places, especially within loops over dynamic data.

2.  **Static Branch Pre-Evaluation**:
    - **Description**: Before entering the main evaluation loop (especially the costly dynamic cross-product evaluation), analyze the top-level expression to identify any major branches that depend _only_ on static data.
    - **Example**: In an expression `(STATIC_CHECK_A AND DYNAMIC_CHECK_B)`, if `STATIC_CHECK_A` can be evaluated to `false` upfront using only `static_data`, the entire evaluation can be short-circuited immediately without iterating through any dynamic data.
    - **Expected Impact**: Provides a powerful mechanism to bypass the most performance-intensive part of the evaluation when static conditions are not met, offering massive speedups for certain data patterns.

## Future Directions

Beyond these phases, several larger architectural changes could be considered for future major versions:

- **Register-Based VM**: A transition from the current stack-based VM to a register-based model could further reduce instruction count and memory access by eliminating many of the `Push` and `Pop` operations.
- **Just-In-Time (JIT) Compilation**: For the ultimate performance in long-running applications, a JIT compiler could translate the Hantei bytecode or AST directly into native machine code at runtime.
