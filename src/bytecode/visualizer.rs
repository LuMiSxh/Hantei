use super::opcode::OpCode;
use std::fmt::Write;

pub fn visualize_bytecode(bytecode: &[OpCode], name: &str) -> String {
    let mut output = String::new();
    writeln!(&mut output, "--- Bytecode for: {} ---", name).unwrap();

    for (i, op) in bytecode.iter().enumerate() {
        let line = format!("{:04}: ", i); // Address/line number
        let op_str = match op {
            OpCode::Jump(addr) | OpCode::JumpIfFalse(addr) => {
                format!(
                    "{:<15} -> {:04}",
                    format!("{:?}", op).split('(').next().unwrap(),
                    addr
                )
            }
            _ => format!("{:<20}", format!("{:?}", op)),
        };
        writeln!(&mut output, "{}{}", line, op_str).unwrap();
    }
    output
}
