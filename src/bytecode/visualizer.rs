use super::{compiler::BytecodeProgram, opcode::OpCode};
use std::fmt::Write;

/// Formats a complete `BytecodeProgram` into a human-readable string for debugging.
pub fn visualize_program(program: &BytecodeProgram, name: &str) -> String {
    let mut output = String::new();
    writeln!(
        &mut output,
        "======== BYTECODE PROGRAM for Quality: {} ========",
        name
    )
    .unwrap();

    // 1. Print the MAIN execution block.
    writeln!(&mut output, "\n--- MAIN ---").unwrap();
    format_bytecode_chunk(&mut output, &program.main);

    // 2. Print all subroutines, sorted by ID for consistent ordering.
    if !program.subroutines.is_empty() {
        writeln!(&mut output, "\n--- SUBROUTINES ---").unwrap();
        let mut sorted_subroutines: Vec<_> = program.subroutines.iter().collect();
        sorted_subroutines.sort_by_key(|(id, _)| **id);

        for (id, bytecode) in sorted_subroutines {
            writeln!(&mut output, "\n--- SUBROUTINE #{} ---", id).unwrap();
            format_bytecode_chunk(&mut output, bytecode);
        }
    }

    writeln!(
        &mut output,
        "\n================ END OF PROGRAM ================"
    )
    .unwrap();
    output
}

/// Helper function to format a single `Vec<OpCode>`.
fn format_bytecode_chunk(output: &mut String, bytecode: &[OpCode]) {
    for (i, op) in bytecode.iter().enumerate() {
        let line = format!("{:04}: ", i);
        let op_str = match op {
            // Format jumps to show the target address within the current chunk.
            OpCode::Jump(addr) | OpCode::JumpIfFalse(addr) | OpCode::JumpIfTrue(addr) => {
                format!(
                    "{:<20} -> {:04}",
                    format!("{:?}", op).split('(').next().unwrap(),
                    addr
                )
            }
            // Format calls to show the target subroutine ID.
            OpCode::Call(id) => {
                format!(
                    "{:<20} -> SUB #{}",
                    format!("{:?}", op).split('(').next().unwrap(),
                    id
                )
            }
            // Default formatting for all other opcodes.
            _ => format!("{:?}", op),
        };
        writeln!(output, "{}{}", line, op_str).unwrap();
    }
}
