#!/bin/bash

# This script runs the main Hantei evaluator CLI with sample data.
# Pass in extra arguments to the Hantei CLI.

cargo run --release --bin hantei-cli --features hantei-cli -- --human $@
