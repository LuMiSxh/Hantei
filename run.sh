#!/bin/bash

# This script runs the main Hantei evaluator CLI with sample data.
# Add the --write-debug-files flag to generate AST text files.

cargo run --release --bin hantei-cli --features hantei-cli -- \
    data/flow.json \
    data/qualities_becker.json \
    data/sample_data.json \
    "$@" # Allows passing extra arguments like --write-debug-files
