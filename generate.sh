#!/bin/bash

# This script runs the data generator tool.
# Pass any arguments to customize the generation process.

OUTPUT_FILE=${1:-generated_data.json}

cargo run --release --bin data-gen --features data-gen -- $@
