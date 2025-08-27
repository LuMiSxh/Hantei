@echo off
REM This script runs the data generator tool.
REM Pass a filename as an argument, e.g., generate_data.bat my_data.json

set OUTPUT_FILE=%1
if [%1]==[] set OUTPUT_FILE=generated_data.json

cargo run --release --bin data-gen --features data-gen -- --output %OUTPUT_FILE%
