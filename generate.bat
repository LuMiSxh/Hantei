@echo off
REM This script runs the data generator tool.
REM Pass any arguments to the tool after the script name.


cargo run --release --bin data-gen --features data-gen -- %*
