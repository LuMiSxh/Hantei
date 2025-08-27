@echo off
REM This script runs the main Hantei evaluator CLI with sample data.
REM Add the --write-debug-files flag to generate AST text files.

cargo run --release --bin hantei --features hantei-cli -- ^
    data/flow.json ^
    data/qualities_becker.json ^
    data/sample_data.json ^
    %*
