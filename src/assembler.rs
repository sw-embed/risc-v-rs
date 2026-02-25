//! RV32 Assembly parser
//!
//! This module provides assembly parsing for a simplified RISC-V instruction set.

use crate::cpu::instruction::Instruction;
use std::collections::HashMap;
use thiserror::Error;

/// Assembly errors
#[derive(Debug, Error, Clone)]
pub enum AssemblyError {
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),

    #[error("Invalid register: {0}")]
    InvalidRegister(String),

    #[error("Invalid operand: {0}")]
    InvalidOperand(String),

    #[error("Undefined label: {0}")]
    UndefinedLabel(String),

    #[error("Parse error on line {line}: {message}")]
    ParseError { line: usize, message: String },
}

/// Assembled output with instructions and disassembly
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyOutput {
    pub instructions: Vec<Instruction>,
    pub disassembly: Vec<String>,
}

/// Assemble RV32 assembly source code
pub fn assemble(source: &str) -> Result<AssemblyOutput, AssemblyError> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut current_address: usize = 0;

    // First pass: collect labels
    for line in source.lines() {
        let line = strip_comments(line).trim();
        if line.is_empty() {
            continue;
        }

        // Check for label (ends with :)
        if let Some(label_end) = line.find(':') {
            let label = line[..label_end].trim().to_uppercase();
            labels.insert(label, current_address);

            // Check if there's an instruction on the same line
            let rest = line[label_end + 1..].trim();
            if !rest.is_empty() {
                current_address += 1; // Each instruction is 1 slot
            }
        } else {
            // Regular instruction
            current_address += 1;
        }
    }

    // Second pass: assemble instructions
    let mut instructions = Vec::new();
    let mut disassembly = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line = strip_comments(line).trim();
        if line.is_empty() {
            continue;
        }

        // Skip label part if present, get instruction
        let instruction_line = if let Some(label_end) = line.find(':') {
            line[label_end + 1..].trim()
        } else {
            line
        };

        if instruction_line.is_empty() {
            continue;
        }

        match assemble_instruction(instruction_line, &labels, instructions.len()) {
            Ok(inst) => {
                // Use byte address (instruction_index * 4) to match PC
                let byte_addr = instructions.len() * 4;
                let disasm = format!("{:04X}: {} | {}", byte_addr, inst, instruction_line);
                disassembly.push(disasm);
                instructions.push(inst);
            }
            Err(e) => {
                return Err(AssemblyError::ParseError {
                    line: line_num + 1,
                    message: e.to_string(),
                });
            }
        }
    }

    Ok(AssemblyOutput {
        instructions,
        disassembly,
    })
}

/// Strip comments from a line (everything after ; or #)
fn strip_comments(line: &str) -> &str {
    if let Some(idx) = line.find(';').or_else(|| line.find('#')) {
        &line[..idx]
    } else {
        line
    }
}

/// Assemble a single instruction
fn assemble_instruction(
    line: &str,
    labels: &HashMap<String, usize>,
    current_address: usize,
) -> Result<Instruction, AssemblyError> {
    let parts: Vec<&str> = line
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Ok(Instruction::Nop);
    }

    let mnemonic = parts[0].to_uppercase();

    match mnemonic.as_str() {
        // R-Type: ADD rd, rs1, rs2
        "ADD" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "ADD requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::Add {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }
        "SUB" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "SUB requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::Sub {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }
        "AND" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "AND requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::And {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }
        "OR" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "OR requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::Or {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }
        "XOR" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "XOR requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::Xor {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }
        "SLT" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "SLT requires 3 operands: rd, rs1, rs2".to_string(),
                ));
            }
            Ok(Instruction::Slt {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                rs2: parse_register(parts[3])?,
            })
        }

        // I-Type: ADDI rd, rs1, imm
        "ADDI" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "ADDI requires 3 operands: rd, rs1, imm".to_string(),
                ));
            }
            Ok(Instruction::Addi {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                imm: parse_immediate(parts[3])?,
            })
        }
        "ANDI" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "ANDI requires 3 operands: rd, rs1, imm".to_string(),
                ));
            }
            Ok(Instruction::Andi {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                imm: parse_immediate(parts[3])?,
            })
        }
        "ORI" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "ORI requires 3 operands: rd, rs1, imm".to_string(),
                ));
            }
            Ok(Instruction::Ori {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                imm: parse_immediate(parts[3])?,
            })
        }
        "XORI" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "XORI requires 3 operands: rd, rs1, imm".to_string(),
                ));
            }
            Ok(Instruction::Xori {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                imm: parse_immediate(parts[3])?,
            })
        }
        "SLTI" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "SLTI requires 3 operands: rd, rs1, imm".to_string(),
                ));
            }
            Ok(Instruction::Slti {
                rd: parse_register(parts[1])?,
                rs1: parse_register(parts[2])?,
                imm: parse_immediate(parts[3])?,
            })
        }

        // Load: LW rd, offset(rs1)
        "LW" | "LB" => {
            if parts.len() < 3 {
                return Err(AssemblyError::InvalidOperand(format!(
                    "{} requires format: rd, offset(rs1)",
                    mnemonic
                )));
            }

            // Parse offset(rs1) format
            let (offset, rs1) = parse_offset_register(parts[2])?;

            if mnemonic == "LW" {
                Ok(Instruction::Lw {
                    rd: parse_register(parts[1])?,
                    rs1,
                    offset,
                })
            } else {
                Ok(Instruction::Lb {
                    rd: parse_register(parts[1])?,
                    rs1,
                    offset,
                })
            }
        }

        // Store: SW rs2, offset(rs1)
        "SW" | "SB" => {
            if parts.len() < 3 {
                return Err(AssemblyError::InvalidOperand(format!(
                    "{} requires format: rs2, offset(rs1)",
                    mnemonic
                )));
            }

            // Parse offset(rs1) format
            let (offset, rs1) = parse_offset_register(parts[2])?;

            if mnemonic == "SW" {
                Ok(Instruction::Sw {
                    rs2: parse_register(parts[1])?,
                    rs1,
                    offset,
                })
            } else {
                Ok(Instruction::Sb {
                    rs2: parse_register(parts[1])?,
                    rs1,
                    offset,
                })
            }
        }

        // Branch: BEQ rs1, rs2, offset
        "BEQ" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "BEQ requires 3 operands: rs1, rs2, offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[3], labels, current_address)?;
            Ok(Instruction::Beq {
                rs1: parse_register(parts[1])?,
                rs2: parse_register(parts[2])?,
                offset,
            })
        }
        "BNE" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "BNE requires 3 operands: rs1, rs2, offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[3], labels, current_address)?;
            Ok(Instruction::Bne {
                rs1: parse_register(parts[1])?,
                rs2: parse_register(parts[2])?,
                offset,
            })
        }
        "BLT" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "BLT requires 3 operands: rs1, rs2, offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[3], labels, current_address)?;
            Ok(Instruction::Blt {
                rs1: parse_register(parts[1])?,
                rs2: parse_register(parts[2])?,
                offset,
            })
        }
        "BGE" => {
            if parts.len() < 4 {
                return Err(AssemblyError::InvalidOperand(
                    "BGE requires 3 operands: rs1, rs2, offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[3], labels, current_address)?;
            Ok(Instruction::Bge {
                rs1: parse_register(parts[1])?,
                rs2: parse_register(parts[2])?,
                offset,
            })
        }

        // Jump: J offset
        "J" => {
            if parts.len() < 2 {
                return Err(AssemblyError::InvalidOperand(
                    "J requires 1 operand: offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[1], labels, current_address)?;
            Ok(Instruction::J { offset })
        }
        "JAL" => {
            if parts.len() < 3 {
                return Err(AssemblyError::InvalidOperand(
                    "JAL requires 2 operands: rd, offset".to_string(),
                ));
            }
            let offset = resolve_branch_target(parts[2], labels, current_address)?;
            Ok(Instruction::Jal {
                rd: parse_register(parts[1])?,
                offset,
            })
        }
        "JALR" => {
            if parts.len() < 3 {
                return Err(AssemblyError::InvalidOperand(
                    "JALR requires format: rd, offset(rs1)".to_string(),
                ));
            }
            let (offset, rs1) = parse_offset_register(parts[2])?;
            Ok(Instruction::Jalr {
                rd: parse_register(parts[1])?,
                rs1,
                offset,
            })
        }

        // Special
        "NOP" => Ok(Instruction::Nop),
        "HALT" => Ok(Instruction::Halt),

        _ => Err(AssemblyError::InvalidInstruction(mnemonic)),
    }
}

/// Parse a register (x0-x7)
fn parse_register(s: &str) -> Result<u8, AssemblyError> {
    let s = s.trim().to_lowercase();

    // Remove 'x' prefix if present
    let num_str = if let Some(stripped) = s.strip_prefix('x') {
        stripped
    } else {
        &s
    };

    // Parse as decimal (0-7)
    let reg = num_str
        .parse::<u8>()
        .map_err(|_| AssemblyError::InvalidRegister(format!("Invalid register: {}", s)))?;

    if reg > 7 {
        return Err(AssemblyError::InvalidRegister(format!(
            "Register must be x0-x7, got: {}",
            s
        )));
    }

    Ok(reg)
}

/// Parse an immediate value
fn parse_immediate(s: &str) -> Result<i32, AssemblyError> {
    let s = s.trim();

    // Hex with 0x prefix
    if s.starts_with("0x") || s.starts_with("0X") {
        i32::from_str_radix(&s[2..], 16)
            .map_err(|_| AssemblyError::InvalidOperand(format!("Invalid hex number: {}", s)))
    }
    // Hex with $ prefix
    else if let Some(stripped) = s.strip_prefix('$') {
        i32::from_str_radix(stripped, 16)
            .map_err(|_| AssemblyError::InvalidOperand(format!("Invalid hex number: {}", s)))
    }
    // Decimal (including negative)
    else {
        s.parse::<i32>()
            .map_err(|_| AssemblyError::InvalidOperand(format!("Invalid number: {}", s)))
    }
}

/// Parse offset(register) format for load/store/JALR
fn parse_offset_register(s: &str) -> Result<(i32, u8), AssemblyError> {
    // Find opening and closing parentheses
    let open = s.find('(').ok_or_else(|| {
        AssemblyError::InvalidOperand(format!("Expected format: offset(reg), got: {}", s))
    })?;

    let close = s.find(')').ok_or_else(|| {
        AssemblyError::InvalidOperand(format!("Expected format: offset(reg), got: {}", s))
    })?;

    let offset_str = s[..open].trim();
    let reg_str = s[open + 1..close].trim();

    let offset = if offset_str.is_empty() {
        0
    } else {
        parse_immediate(offset_str)?
    };

    let reg = parse_register(reg_str)?;

    Ok((offset, reg))
}

/// Resolve a branch target (label or immediate offset)
fn resolve_branch_target(
    target: &str,
    labels: &HashMap<String, usize>,
    current_address: usize,
) -> Result<i32, AssemblyError> {
    let target_upper = target.trim().to_uppercase();

    // Check if it's a label
    if let Some(&label_addr) = labels.get(&target_upper) {
        // Calculate relative offset from current instruction
        let offset = (label_addr as i32) - (current_address as i32);
        Ok(offset)
    } else {
        // Try to parse as immediate
        parse_immediate(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("x0").unwrap(), 0);
        assert_eq!(parse_register("x7").unwrap(), 7);
        assert_eq!(parse_register("X5").unwrap(), 5);
        assert!(parse_register("x8").is_err()); // Out of range
    }

    #[test]
    fn test_parse_immediate() {
        assert_eq!(parse_immediate("42").unwrap(), 42);
        assert_eq!(parse_immediate("-10").unwrap(), -10);
        assert_eq!(parse_immediate("0x10").unwrap(), 16);
        assert_eq!(parse_immediate("$FF").unwrap(), 255);
    }

    #[test]
    fn test_parse_offset_register() {
        let (offset, reg) = parse_offset_register("4(x2)").unwrap();
        assert_eq!(offset, 4);
        assert_eq!(reg, 2);

        let (offset, reg) = parse_offset_register("(x5)").unwrap();
        assert_eq!(offset, 0);
        assert_eq!(reg, 5);
    }

    #[test]
    fn test_assemble_addi() {
        let source = "ADDI x1, x0, 5";
        let result = assemble(source).unwrap();
        assert_eq!(result.instructions.len(), 1);
        match result.instructions[0] {
            Instruction::Addi { rd, rs1, imm } => {
                assert_eq!(rd, 1);
                assert_eq!(rs1, 0);
                assert_eq!(imm, 5);
            }
            _ => panic!("Expected ADDI instruction"),
        }
    }

    #[test]
    fn test_assemble_add() {
        let source = "ADD x3, x1, x2";
        let result = assemble(source).unwrap();
        assert_eq!(result.instructions.len(), 1);
        match result.instructions[0] {
            Instruction::Add { rd, rs1, rs2 } => {
                assert_eq!(rd, 3);
                assert_eq!(rs1, 1);
                assert_eq!(rs2, 2);
            }
            _ => panic!("Expected ADD instruction"),
        }
    }

    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
START: ADDI x1, x0, 5
       ADDI x2, x0, 10
       BEQ x1, x2, START
"#;
        let result = assemble(source).unwrap();
        assert_eq!(result.instructions.len(), 3);

        // Third instruction should be BEQ with offset -2 (back to START)
        match result.instructions[2] {
            Instruction::Beq { rs1, rs2, offset } => {
                assert_eq!(rs1, 1);
                assert_eq!(rs2, 2);
                assert_eq!(offset, -2); // Jump back 2 instructions
            }
            _ => panic!("Expected BEQ instruction"),
        }
    }

    #[test]
    fn test_assemble_load_store() {
        let source = r#"
LW x1, 4(x2)
SW x3, 8(x4)
"#;
        let result = assemble(source).unwrap();
        assert_eq!(result.instructions.len(), 2);

        match result.instructions[0] {
            Instruction::Lw { rd, rs1, offset } => {
                assert_eq!(rd, 1);
                assert_eq!(rs1, 2);
                assert_eq!(offset, 4);
            }
            _ => panic!("Expected LW instruction"),
        }

        match result.instructions[1] {
            Instruction::Sw { rs2, rs1, offset } => {
                assert_eq!(rs2, 3);
                assert_eq!(rs1, 4);
                assert_eq!(offset, 8);
            }
            _ => panic!("Expected SW instruction"),
        }
    }

    #[test]
    fn test_comments() {
        let source = r#"
; This is a comment
ADDI x1, x0, 5  # Load 5 into x1
HALT
"#;
        let result = assemble(source).unwrap();
        assert_eq!(result.instructions.len(), 2);
    }
}
