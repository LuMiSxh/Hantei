@echo off
REM This script runs the main Hantei evaluator CLI with sample data.
REM Pass in extra arguments to the Hantei CLI if needed.

cargo run --release --bin hantei-cli --features hantei-cli -- --human %*
