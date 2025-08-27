#!/bin/bash

# This script runs the data generator tool.
# Pass a filename as an argument, e.g., ./generate_data.sh my_data.json

OUTPUT_FILE=${1:-generated_data.json}

cargo run --release --bin data-gen --features data-gen -- --output "$OUTPUT_FILE"
