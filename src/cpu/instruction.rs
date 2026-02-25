//! RISC-V instruction definitions
//!
//! This module defines a simplified RISC-V instruction set for educational purposes.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Register identifier (0-7 for x0-x7)
pub type Register = u8;

/// Immediate value (signed 12-bit for most instructions)
pub type Immediate = i32;

/// Memory address
pub type Address = u32;

/// Instruction formats based on RISC-V types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // ===== Arithmetic (R-Type) =====
    /// ADD rd, rs1, rs2 - rd = rs1 + rs2
    Add {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// SUB rd, rs1, rs2 - rd = rs1 - rs2
    Sub {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// AND rd, rs1, rs2 - rd = rs1 & rs2
    And {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// OR rd, rs1, rs2 - rd = rs1 | rs2
    Or {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// XOR rd, rs1, rs2 - rd = rs1 ^ rs2
    Xor {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// SLT rd, rs1, rs2 - rd = (rs1 < rs2) ? 1 : 0 (signed)
    Slt {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },

    // ===== Immediate (I-Type) =====
    /// ADDI rd, rs1, imm - rd = rs1 + imm
    Addi {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// ANDI rd, rs1, imm - rd = rs1 & imm
    Andi {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// ORI rd, rs1, imm - rd = rs1 | imm
    Ori {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// XORI rd, rs1, imm - rd = rs1 ^ imm
    Xori {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// SLTI rd, rs1, imm - rd = (rs1 < imm) ? 1 : 0 (signed)
    Slti {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },

    // ===== Memory Operations (Load/Store) =====
    /// LW rd, imm(rs1) - rd = Memory[rs1 + imm] (load word - 4 bytes)
    Lw {
        rd: Register,
        rs1: Register,
        offset: Immediate,
    },
    /// LB rd, imm(rs1) - rd = Memory[rs1 + imm] (load byte - 1 byte, sign-extended)
    Lb {
        rd: Register,
        rs1: Register,
        offset: Immediate,
    },
    /// SW rs2, imm(rs1) - Memory[rs1 + imm] = rs2 (store word - 4 bytes)
    Sw {
        rs2: Register,
        rs1: Register,
        offset: Immediate,
    },
    /// SB rs2, imm(rs1) - Memory[rs1 + imm] = rs2 (store byte - 1 byte)
    Sb {
        rs2: Register,
        rs1: Register,
        offset: Immediate,
    },

    // ===== Control Flow (Branch) =====
    /// BEQ rs1, rs2, offset - if (rs1 == rs2) PC += offset
    Beq {
        rs1: Register,
        rs2: Register,
        offset: Immediate,
    },
    /// BNE rs1, rs2, offset - if (rs1 != rs2) PC += offset
    Bne {
        rs1: Register,
        rs2: Register,
        offset: Immediate,
    },
    /// BLT rs1, rs2, offset - if (rs1 < rs2) PC += offset (signed)
    Blt {
        rs1: Register,
        rs2: Register,
        offset: Immediate,
    },
    /// BGE rs1, rs2, offset - if (rs1 >= rs2) PC += offset (signed)
    Bge {
        rs1: Register,
        rs2: Register,
        offset: Immediate,
    },

    // ===== Jump Instructions =====
    /// J offset - PC += offset (unconditional jump)
    J { offset: Immediate },
    /// JAL rd, offset - rd = PC + 4; PC += offset (jump and link)
    Jal { rd: Register, offset: Immediate },
    /// JALR rd, rs1, offset - rd = PC + 4; PC = rs1 + offset
    Jalr {
        rd: Register,
        rs1: Register,
        offset: Immediate,
    },

    // ===== Special =====
    /// NOP - No operation
    Nop,
    /// HALT - Stop execution
    Halt,
}

impl Instruction {
    /// Returns true if this instruction modifies the program counter
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Instruction::Beq { .. }
                | Instruction::Bne { .. }
                | Instruction::Blt { .. }
                | Instruction::Bge { .. }
                | Instruction::J { .. }
                | Instruction::Jal { .. }
                | Instruction::Jalr { .. }
        )
    }

    /// Returns true if this instruction accesses memory
    pub fn is_memory_access(&self) -> bool {
        matches!(
            self,
            Instruction::Lw { .. }
                | Instruction::Lb { .. }
                | Instruction::Sw { .. }
                | Instruction::Sb { .. }
        )
    }

    /// Returns the mnemonic (instruction name) as a string
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Instruction::Add { .. } => "ADD",
            Instruction::Sub { .. } => "SUB",
            Instruction::And { .. } => "AND",
            Instruction::Or { .. } => "OR",
            Instruction::Xor { .. } => "XOR",
            Instruction::Slt { .. } => "SLT",
            Instruction::Addi { .. } => "ADDI",
            Instruction::Andi { .. } => "ANDI",
            Instruction::Ori { .. } => "ORI",
            Instruction::Xori { .. } => "XORI",
            Instruction::Slti { .. } => "SLTI",
            Instruction::Lw { .. } => "LW",
            Instruction::Lb { .. } => "LB",
            Instruction::Sw { .. } => "SW",
            Instruction::Sb { .. } => "SB",
            Instruction::Beq { .. } => "BEQ",
            Instruction::Bne { .. } => "BNE",
            Instruction::Blt { .. } => "BLT",
            Instruction::Bge { .. } => "BGE",
            Instruction::J { .. } => "J",
            Instruction::Jal { .. } => "JAL",
            Instruction::Jalr { .. } => "JALR",
            Instruction::Nop => "NOP",
            Instruction::Halt => "HALT",
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // R-Type
            Instruction::Add { rd, rs1, rs2 }
            | Instruction::Sub { rd, rs1, rs2 }
            | Instruction::And { rd, rs1, rs2 }
            | Instruction::Or { rd, rs1, rs2 }
            | Instruction::Xor { rd, rs1, rs2 }
            | Instruction::Slt { rd, rs1, rs2 } => {
                write!(f, "{} x{}, x{}, x{}", self.mnemonic(), rd, rs1, rs2)
            }
            // I-Type Arithmetic
            Instruction::Addi { rd, rs1, imm }
            | Instruction::Andi { rd, rs1, imm }
            | Instruction::Ori { rd, rs1, imm }
            | Instruction::Xori { rd, rs1, imm }
            | Instruction::Slti { rd, rs1, imm } => {
                write!(f, "{} x{}, x{}, {}", self.mnemonic(), rd, rs1, imm)
            }
            // Load
            Instruction::Lw { rd, rs1, offset } | Instruction::Lb { rd, rs1, offset } => {
                write!(f, "{} x{}, {}(x{})", self.mnemonic(), rd, offset, rs1)
            }
            // Store
            Instruction::Sw { rs2, rs1, offset } | Instruction::Sb { rs2, rs1, offset } => {
                write!(f, "{} x{}, {}(x{})", self.mnemonic(), rs2, offset, rs1)
            }
            // Branch
            Instruction::Beq { rs1, rs2, offset }
            | Instruction::Bne { rs1, rs2, offset }
            | Instruction::Blt { rs1, rs2, offset }
            | Instruction::Bge { rs1, rs2, offset } => {
                write!(f, "{} x{}, x{}, {}", self.mnemonic(), rs1, rs2, offset)
            }
            // Jump
            Instruction::J { offset } => write!(f, "{} {}", self.mnemonic(), offset),
            Instruction::Jal { rd, offset } => {
                write!(f, "{} x{}, {}", self.mnemonic(), rd, offset)
            }
            Instruction::Jalr { rd, rs1, offset } => {
                write!(f, "{} x{}, {}(x{})", self.mnemonic(), rd, offset, rs1)
            }
            // Special
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Halt => write!(f, "HALT"),
        }
    }
}
