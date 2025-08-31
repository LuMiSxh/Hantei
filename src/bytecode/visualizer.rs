use super::{compiler::BytecodeProgram, opcode::OpCode};
use crate::ast::InputId;
use ahash::AHashMap;
use std::fmt::Write;

/// Formats a complete `BytecodeProgram` into a human-readable string for debugging.
pub fn visualize_program(
    program: &BytecodeProgram,
    name: &str,
    static_rev_map: &AHashMap<InputId, String>,
    dynamic_rev_map: &AHashMap<InputId, String>,
) -> String {
    let mut output = String::new();
    writeln!(
        &mut output,
        "======== BYTECODE PROGRAM for Quality: {} ========",
        name
    )
    .unwrap();

    if !program.main.is_empty() {
        writeln!(&mut output, "\n--- MAIN ---").unwrap();
        format_bytecode_chunk(&mut output, &program.main, static_rev_map, dynamic_rev_map);
    }

    if !program.subroutines.is_empty() {
        writeln!(&mut output, "\n--- SUBROUTINES ---").unwrap();
        let mut sorted_subroutines: Vec<_> = program.subroutines.iter().collect();
        sorted_subroutines.sort_by_key(|(id, _)| **id);

        for (id, bytecode) in sorted_subroutines {
            writeln!(&mut output, "\n--- SUBROUTINE #{} ---", id).unwrap();
            format_bytecode_chunk(&mut output, bytecode, static_rev_map, dynamic_rev_map);
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
fn format_bytecode_chunk(
    output: &mut String,
    bytecode: &[OpCode],
    static_rev_map: &AHashMap<InputId, String>,
    dynamic_rev_map: &AHashMap<InputId, String>,
) {
    for (i, op) in bytecode.iter().enumerate() {
        let line = format!("{:04}: ", i);
        let op_str = match op {
            OpCode::LoadStatic(r, id) => {
                let name = static_rev_map.get(id).map(|s| s.as_str()).unwrap_or("?");
                format!("{:<20} R{}, ${} [S{}]", "LoadStatic", r, name, id)
            }
            OpCode::LoadDynamic(r, id) => {
                let name = dynamic_rev_map.get(id).map(|s| s.as_str()).unwrap_or("?");
                format!("{:<20} R{}, ${} [D{}]", "LoadDynamic", r, name, id)
            }
            // --- Standard formatting for other opcodes ---
            OpCode::LoadLiteral(r, v) => format!("{:<20} R{}, {}", "LoadLiteral", r, v),
            OpCode::Move(d, s) => format!("{:<20} R{}, R{}", "Move", d, s),
            OpCode::Add(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Add", d, s1, s2),
            OpCode::Subtract(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Subtract", d, s1, s2),
            OpCode::Multiply(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Multiply", d, s1, s2),
            OpCode::Divide(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Divide", d, s1, s2),
            OpCode::Xor(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Xor", d, s1, s2),
            OpCode::Abs(d, s) => format!("{:<20} R{}, R{}", "Abs", d, s),
            OpCode::Not(d, s) => format!("{:<20} R{}, R{}", "Not", d, s),
            OpCode::Equal(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "Equal", d, s1, s2),
            OpCode::NotEqual(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "NotEqual", d, s1, s2),
            OpCode::GreaterThan(d, s1, s2) => {
                format!("{:<20} R{}, R{}, R{}", "GreaterThan", d, s1, s2)
            }
            OpCode::LessThan(d, s1, s2) => format!("{:<20} R{}, R{}, R{}", "LessThan", d, s1, s2),
            OpCode::GreaterThanOrEqual(d, s1, s2) => {
                format!("{:<20} R{}, R{}, R{}", "GtOrEqual", d, s1, s2)
            }
            OpCode::LessThanOrEqual(d, s1, s2) => {
                format!("{:<20} R{}, R{}, R{}", "LtOrEqual", d, s1, s2)
            }
            OpCode::JumpIfEq(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfEq", r1, r2, addr)
            }
            OpCode::JumpIfNeq(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfNeq", r1, r2, addr)
            }
            OpCode::JumpIfGt(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfGt", r1, r2, addr)
            }
            OpCode::JumpIfGte(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfGte", r1, r2, addr)
            }
            OpCode::JumpIfLt(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfLt", r1, r2, addr)
            }
            OpCode::JumpIfLte(r1, r2, addr) => {
                format!("{:<20} R{}, R{}, -> {:04}", "JumpIfLte", r1, r2, addr)
            }
            OpCode::Jump(addr) => format!("{:<20} -> {:04}", "Jump", addr),
            OpCode::JumpIfFalse(r, addr) => format!("{:<20} R{}, -> {:04}", "JumpIfFalse", r, addr),
            OpCode::JumpIfTrue(r, addr) => format!("{:<20} R{}, -> {:04}", "JumpIfTrue", r, addr),
            OpCode::Call(id) => format!("{:<20} -> SUB #{}", "Call", id),
            OpCode::Return => "Return".to_string(),
            OpCode::Halt => "Halt".to_string(),
        };
        writeln!(output, "{}{}", line, op_str).unwrap();
    }
}
