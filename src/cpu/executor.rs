//! Instruction execution engine
//!
//! This module implements the execution logic for all RISC-V instructions.

use super::instruction::Instruction;
use super::state::{CpuError, CpuState};

/// Execute a single instruction on the CPU
///
/// Returns true if execution should continue, false if CPU is halted
pub fn execute(cpu: &mut CpuState, instruction: Instruction) -> Result<bool, CpuError> {
    if cpu.is_halted() {
        return Err(CpuError::Halted);
    }

    cpu.count_instruction();

    match instruction {
        // ===== Arithmetic (R-Type) =====
        Instruction::Add { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;
            let result = a.wrapping_add(b);

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.set_add_flags(a, b, result);
            cpu.increment_pc()?;
        }

        Instruction::Sub { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;
            let result = a.wrapping_sub(b);

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.set_sub_flags(a, b, result);
            cpu.increment_pc()?;
        }

        Instruction::And { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;
            let result = a & b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Or { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;
            let result = a | b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Xor { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;
            let result = a ^ b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Slt { rd, rs1, rs2 } => {
            let a = cpu.read_register(rs1)? as i32;
            let b = cpu.read_register(rs2)? as i32;
            let result = if a < b { 1 } else { 0 };

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        // ===== Immediate (I-Type) =====
        Instruction::Addi { rd, rs1, imm } => {
            let a = cpu.read_register(rs1)?;
            let b = imm as u32;
            let result = a.wrapping_add(b);

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.set_add_flags(a, b, result);
            cpu.increment_pc()?;
        }

        Instruction::Andi { rd, rs1, imm } => {
            let a = cpu.read_register(rs1)?;
            let b = imm as u32;
            let result = a & b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Ori { rd, rs1, imm } => {
            let a = cpu.read_register(rs1)?;
            let b = imm as u32;
            let result = a | b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Xori { rd, rs1, imm } => {
            let a = cpu.read_register(rs1)?;
            let b = imm as u32;
            let result = a ^ b;

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        Instruction::Slti { rd, rs1, imm } => {
            let a = cpu.read_register(rs1)? as i32;
            let b = imm;
            let result = if a < b { 1 } else { 0 };

            cpu.write_register(rd, result)?;
            cpu.update_flags(result);
            cpu.increment_pc()?;
        }

        // ===== Memory Operations =====
        Instruction::Lw { rd, rs1, offset } => {
            let base = cpu.read_register(rs1)?;
            let addr = (base as i32).wrapping_add(offset) as u32;
            let value = cpu.read_word(addr)?;

            cpu.write_register(rd, value)?;
            cpu.increment_pc()?;
        }

        Instruction::Lb { rd, rs1, offset } => {
            let base = cpu.read_register(rs1)?;
            let addr = (base as i32).wrapping_add(offset) as u32;
            let byte = cpu.read_byte(addr)?;
            // Sign-extend byte to 32 bits
            let value = (byte as i8) as i32 as u32;

            cpu.write_register(rd, value)?;
            cpu.increment_pc()?;
        }

        Instruction::Sw { rs2, rs1, offset } => {
            let base = cpu.read_register(rs1)?;
            let addr = (base as i32).wrapping_add(offset) as u32;
            let value = cpu.read_register(rs2)?;

            cpu.write_word(addr, value)?;
            cpu.increment_pc()?;
        }

        Instruction::Sb { rs2, rs1, offset } => {
            let base = cpu.read_register(rs1)?;
            let addr = (base as i32).wrapping_add(offset) as u32;
            let value = cpu.read_register(rs2)? as u8;

            cpu.write_byte(addr, value)?;
            cpu.increment_pc()?;
        }

        // ===== Control Flow (Branches) =====
        Instruction::Beq { rs1, rs2, offset } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;

            if a == b {
                let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
                cpu.set_pc(new_pc)?;
            } else {
                cpu.increment_pc()?;
            }
        }

        Instruction::Bne { rs1, rs2, offset } => {
            let a = cpu.read_register(rs1)?;
            let b = cpu.read_register(rs2)?;

            if a != b {
                let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
                cpu.set_pc(new_pc)?;
            } else {
                cpu.increment_pc()?;
            }
        }

        Instruction::Blt { rs1, rs2, offset } => {
            let a = cpu.read_register(rs1)? as i32;
            let b = cpu.read_register(rs2)? as i32;

            if a < b {
                let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
                cpu.set_pc(new_pc)?;
            } else {
                cpu.increment_pc()?;
            }
        }

        Instruction::Bge { rs1, rs2, offset } => {
            let a = cpu.read_register(rs1)? as i32;
            let b = cpu.read_register(rs2)? as i32;

            if a >= b {
                let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
                cpu.set_pc(new_pc)?;
            } else {
                cpu.increment_pc()?;
            }
        }

        // ===== Jump Instructions =====
        Instruction::J { offset } => {
            let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
            cpu.set_pc(new_pc)?;
        }

        Instruction::Jal { rd, offset } => {
            // Save return address (PC + 4)
            let return_addr = cpu.pc() + 4;
            cpu.write_register(rd, return_addr)?;

            // Jump to target
            let new_pc = (cpu.pc() as i32).wrapping_add(offset) as u32;
            cpu.set_pc(new_pc)?;
        }

        Instruction::Jalr { rd, rs1, offset } => {
            // Save return address (PC + 4)
            let return_addr = cpu.pc() + 4;

            // Calculate target address
            let base = cpu.read_register(rs1)?;
            let target = (base as i32).wrapping_add(offset) as u32;

            // Write return address first (in case rd == rs1)
            cpu.write_register(rd, return_addr)?;

            // Jump to target
            cpu.set_pc(target)?;
        }

        // ===== Special =====
        Instruction::Nop => {
            cpu.increment_pc()?;
        }

        Instruction::Halt => {
            cpu.halt();
            return Ok(false); // Stop execution
        }
    }

    cpu.tick(); // Increment cycle counter
    Ok(!cpu.is_halted())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 10).unwrap();
        cpu.write_register(2, 20).unwrap();

        let inst = Instruction::Add {
            rd: 3,
            rs1: 1,
            rs2: 2,
        };
        execute(&mut cpu, inst).unwrap();

        assert_eq!(cpu.read_register(3).unwrap(), 30);
        assert_eq!(cpu.pc(), 4);
        assert_eq!(cpu.instruction_count(), 1);
    }

    #[test]
    fn test_sub() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 50).unwrap();
        cpu.write_register(2, 20).unwrap();

        let inst = Instruction::Sub {
            rd: 3,
            rs1: 1,
            rs2: 2,
        };
        execute(&mut cpu, inst).unwrap();

        assert_eq!(cpu.read_register(3).unwrap(), 30);
        assert!(!cpu.flags().zero);
    }

    #[test]
    fn test_addi() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 10).unwrap();

        let inst = Instruction::Addi {
            rd: 2,
            rs1: 1,
            imm: 5,
        };
        execute(&mut cpu, inst).unwrap();

        assert_eq!(cpu.read_register(2).unwrap(), 15);
    }

    #[test]
    fn test_logical_ops() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 0b1100).unwrap();
        cpu.write_register(2, 0b1010).unwrap();

        // AND
        execute(
            &mut cpu,
            Instruction::And {
                rd: 3,
                rs1: 1,
                rs2: 2,
            },
        )
        .unwrap();
        assert_eq!(cpu.read_register(3).unwrap(), 0b1000);

        // OR
        cpu.set_pc(0).unwrap();
        execute(
            &mut cpu,
            Instruction::Or {
                rd: 4,
                rs1: 1,
                rs2: 2,
            },
        )
        .unwrap();
        assert_eq!(cpu.read_register(4).unwrap(), 0b1110);

        // XOR
        cpu.set_pc(0).unwrap();
        execute(
            &mut cpu,
            Instruction::Xor {
                rd: 5,
                rs1: 1,
                rs2: 2,
            },
        )
        .unwrap();
        assert_eq!(cpu.read_register(5).unwrap(), 0b0110);
    }

    #[test]
    fn test_slt() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 5).unwrap();
        cpu.write_register(2, 10).unwrap();

        execute(
            &mut cpu,
            Instruction::Slt {
                rd: 3,
                rs1: 1,
                rs2: 2,
            },
        )
        .unwrap();
        assert_eq!(cpu.read_register(3).unwrap(), 1);

        cpu.set_pc(0).unwrap();
        execute(
            &mut cpu,
            Instruction::Slt {
                rd: 4,
                rs1: 2,
                rs2: 1,
            },
        )
        .unwrap();
        assert_eq!(cpu.read_register(4).unwrap(), 0);
    }

    #[test]
    fn test_memory_operations() {
        let mut cpu = CpuState::new();

        // Store word
        cpu.write_register(1, 0x100).unwrap(); // Base address
        cpu.write_register(2, 0xDEADBEEF).unwrap(); // Value

        execute(
            &mut cpu,
            Instruction::Sw {
                rs2: 2,
                rs1: 1,
                offset: 0,
            },
        )
        .unwrap();

        // Load word
        cpu.set_pc(0).unwrap();
        execute(
            &mut cpu,
            Instruction::Lw {
                rd: 3,
                rs1: 1,
                offset: 0,
            },
        )
        .unwrap();

        assert_eq!(cpu.read_register(3).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_byte_operations() {
        let mut cpu = CpuState::new();

        // Store byte
        cpu.write_register(1, 0x100).unwrap();
        cpu.write_register(2, 0xFF).unwrap();

        execute(
            &mut cpu,
            Instruction::Sb {
                rs2: 2,
                rs1: 1,
                offset: 0,
            },
        )
        .unwrap();

        // Load byte (sign-extended)
        cpu.set_pc(0).unwrap();
        execute(
            &mut cpu,
            Instruction::Lb {
                rd: 3,
                rs1: 1,
                offset: 0,
            },
        )
        .unwrap();

        assert_eq!(cpu.read_register(3).unwrap(), 0xFFFFFFFF); // -1 sign-extended
    }

    #[test]
    fn test_beq() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 10).unwrap();
        cpu.write_register(2, 10).unwrap();

        // Equal - should branch
        execute(
            &mut cpu,
            Instruction::Beq {
                rs1: 1,
                rs2: 2,
                offset: 12,
            },
        )
        .unwrap();
        assert_eq!(cpu.pc(), 12);

        // Not equal - should not branch
        cpu.set_pc(0).unwrap();
        cpu.write_register(2, 20).unwrap();
        execute(
            &mut cpu,
            Instruction::Beq {
                rs1: 1,
                rs2: 2,
                offset: 12,
            },
        )
        .unwrap();
        assert_eq!(cpu.pc(), 4);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CpuState::new();
        cpu.write_register(1, 10).unwrap();
        cpu.write_register(2, 20).unwrap();

        // Not equal - should branch
        execute(
            &mut cpu,
            Instruction::Bne {
                rs1: 1,
                rs2: 2,
                offset: 8,
            },
        )
        .unwrap();
        assert_eq!(cpu.pc(), 8);
    }

    #[test]
    fn test_jump() {
        let mut cpu = CpuState::new();

        execute(&mut cpu, Instruction::J { offset: 16 }).unwrap();
        assert_eq!(cpu.pc(), 16);
    }

    #[test]
    fn test_jal() {
        let mut cpu = CpuState::new();

        execute(&mut cpu, Instruction::Jal { rd: 1, offset: 20 }).unwrap();

        assert_eq!(cpu.read_register(1).unwrap(), 4); // Return address
        assert_eq!(cpu.pc(), 20); // Jumped
    }

    #[test]
    fn test_jalr() {
        let mut cpu = CpuState::new();
        cpu.write_register(2, 100).unwrap();

        execute(
            &mut cpu,
            Instruction::Jalr {
                rd: 1,
                rs1: 2,
                offset: 4,
            },
        )
        .unwrap();

        assert_eq!(cpu.read_register(1).unwrap(), 4); // Return address
        assert_eq!(cpu.pc(), 104); // Jumped to x2 + 4
    }

    #[test]
    fn test_nop() {
        let mut cpu = CpuState::new();

        execute(&mut cpu, Instruction::Nop).unwrap();

        assert_eq!(cpu.pc(), 4);
        assert_eq!(cpu.instruction_count(), 1);
    }

    #[test]
    fn test_halt() {
        let mut cpu = CpuState::new();

        let should_continue = execute(&mut cpu, Instruction::Halt).unwrap();

        assert!(!should_continue);
        assert!(cpu.is_halted());
    }

    #[test]
    fn test_halted_execution() {
        let mut cpu = CpuState::new();
        cpu.halt();

        let result = execute(
            &mut cpu,
            Instruction::Add {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CpuError::Halted);
    }
}
