//! CPU state management
//!
//! This module implements the CPU state including registers, memory, and flags.

use super::instruction::Register;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Number of general-purpose registers
pub const NUM_REGISTERS: usize = 32;

/// Memory size in bytes (1KB)
pub const MEMORY_SIZE: usize = 1024;

/// Memory regions
pub const CODE_START: u32 = 0x000;
pub const CODE_END: u32 = 0x0FF;
pub const DATA_START: u32 = 0x100;
pub const DATA_END: u32 = 0x1FF;
pub const STACK_START: u32 = 0x200;
pub const STACK_END: u32 = 0x2FF;
pub const HEAP_START: u32 = 0x300;
pub const HEAP_END: u32 = 0x3FF;

/// Initial stack pointer value (top of stack, grows down)
pub const INITIAL_SP: u32 = STACK_END;

/// CPU execution errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CpuError {
    #[error("Invalid register: x{0} (must be 0-31)")]
    InvalidRegister(u8),

    #[error("Memory access out of bounds: 0x{0:X} (size: {1} bytes)")]
    MemoryOutOfBounds(u32, usize),

    #[error("Unaligned memory access: 0x{0:X} (must be aligned to {1} bytes)")]
    UnalignedAccess(u32, usize),

    #[error("CPU is halted")]
    Halted,

    #[error("Program counter out of bounds: 0x{0:X}")]
    PcOutOfBounds(u32),
}

/// CPU status flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatusFlags {
    /// Zero flag: last result was zero
    pub zero: bool,

    /// Negative flag: last result was negative (MSB set)
    pub negative: bool,

    /// Carry flag: unsigned overflow/underflow
    pub carry: bool,

    /// Overflow flag: signed overflow/underflow
    pub overflow: bool,
}

impl fmt::Display for StatusFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Z={} N={} C={} V={}",
            self.zero as u8, self.negative as u8, self.carry as u8, self.overflow as u8
        )
    }
}

/// CPU state
#[derive(Debug, Clone)]
pub struct CpuState {
    /// General-purpose registers (x0-x31)
    /// x0 is hardwired to 0
    /// x2 is conventionally used as stack pointer
    registers: [u32; NUM_REGISTERS],

    /// Program counter
    pc: u32,

    /// Status flags
    flags: StatusFlags,

    /// Memory (1KB)
    memory: [u8; MEMORY_SIZE],

    /// Execution state
    halted: bool,

    /// Cycle counter
    cycle_count: u64,

    /// Instruction counter
    instruction_count: u64,
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuState {
    /// Create a new CPU with default state
    pub fn new() -> Self {
        let mut cpu = Self {
            registers: [0; NUM_REGISTERS],
            pc: CODE_START,
            flags: StatusFlags::default(),
            memory: [0; MEMORY_SIZE],
            halted: false,
            cycle_count: 0,
            instruction_count: 0,
        };

        // Initialize stack pointer (x2)
        cpu.registers[2] = INITIAL_SP;

        cpu
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.registers = [0; NUM_REGISTERS];
        self.registers[2] = INITIAL_SP; // Reset stack pointer
        self.pc = CODE_START;
        self.flags = StatusFlags::default();
        self.halted = false;
        self.cycle_count = 0;
        self.instruction_count = 0;
        // Note: Memory is NOT cleared on reset (program stays loaded)
    }

    /// Reset and clear all memory
    pub fn hard_reset(&mut self) {
        self.reset();
        self.memory = [0; MEMORY_SIZE];
    }

    // ===== Register Access =====

    /// Read a register value
    /// x0 always returns 0
    pub fn read_register(&self, reg: Register) -> Result<u32, CpuError> {
        if reg >= NUM_REGISTERS as u8 {
            return Err(CpuError::InvalidRegister(reg));
        }

        // x0 is hardwired to zero
        if reg == 0 {
            Ok(0)
        } else {
            Ok(self.registers[reg as usize])
        }
    }

    /// Write a register value
    /// Writes to x0 are ignored
    pub fn write_register(&mut self, reg: Register, value: u32) -> Result<(), CpuError> {
        if reg >= NUM_REGISTERS as u8 {
            return Err(CpuError::InvalidRegister(reg));
        }

        // x0 is hardwired to zero (writes ignored)
        if reg != 0 {
            self.registers[reg as usize] = value;
        }

        Ok(())
    }

    /// Get stack pointer value (x2)
    pub fn stack_pointer(&self) -> u32 {
        self.registers[2]
    }

    /// Set stack pointer value (x2)
    pub fn set_stack_pointer(&mut self, sp: u32) {
        self.registers[2] = sp;
    }

    // ===== Memory Access =====

    /// Read a byte from memory
    pub fn read_byte(&self, addr: u32) -> Result<u8, CpuError> {
        if addr >= MEMORY_SIZE as u32 {
            return Err(CpuError::MemoryOutOfBounds(addr, 1));
        }
        Ok(self.memory[addr as usize])
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, addr: u32, value: u8) -> Result<(), CpuError> {
        if addr >= MEMORY_SIZE as u32 {
            return Err(CpuError::MemoryOutOfBounds(addr, 1));
        }
        self.memory[addr as usize] = value;
        Ok(())
    }

    /// Read a word (4 bytes, little-endian) from memory
    pub fn read_word(&self, addr: u32) -> Result<u32, CpuError> {
        // Check alignment
        if !addr.is_multiple_of(4) {
            return Err(CpuError::UnalignedAccess(addr, 4));
        }

        if addr + 3 >= MEMORY_SIZE as u32 {
            return Err(CpuError::MemoryOutOfBounds(addr, 4));
        }

        let addr = addr as usize;
        let bytes = [
            self.memory[addr],
            self.memory[addr + 1],
            self.memory[addr + 2],
            self.memory[addr + 3],
        ];

        Ok(u32::from_le_bytes(bytes))
    }

    /// Write a word (4 bytes, little-endian) to memory
    pub fn write_word(&mut self, addr: u32, value: u32) -> Result<(), CpuError> {
        // Check alignment
        if !addr.is_multiple_of(4) {
            return Err(CpuError::UnalignedAccess(addr, 4));
        }

        if addr + 3 >= MEMORY_SIZE as u32 {
            return Err(CpuError::MemoryOutOfBounds(addr, 4));
        }

        let addr = addr as usize;
        let bytes = value.to_le_bytes();

        self.memory[addr] = bytes[0];
        self.memory[addr + 1] = bytes[1];
        self.memory[addr + 2] = bytes[2];
        self.memory[addr + 3] = bytes[3];

        Ok(())
    }

    /// Load program into memory starting at address
    pub fn load_program(&mut self, addr: u32, data: &[u8]) -> Result<(), CpuError> {
        if addr as usize + data.len() > MEMORY_SIZE {
            return Err(CpuError::MemoryOutOfBounds(addr, data.len()));
        }

        let start = addr as usize;
        self.memory[start..start + data.len()].copy_from_slice(data);
        Ok(())
    }

    // ===== Program Counter =====

    /// Get program counter
    pub fn pc(&self) -> u32 {
        self.pc
    }

    /// Set program counter
    pub fn set_pc(&mut self, pc: u32) -> Result<(), CpuError> {
        if pc >= MEMORY_SIZE as u32 {
            return Err(CpuError::PcOutOfBounds(pc));
        }
        self.pc = pc;
        Ok(())
    }

    /// Increment program counter by 4 (size of one instruction)
    pub fn increment_pc(&mut self) -> Result<(), CpuError> {
        self.set_pc(self.pc + 4)
    }

    // ===== Flags =====

    /// Get status flags
    pub fn flags(&self) -> StatusFlags {
        self.flags
    }

    /// Update flags based on a result value
    pub fn update_flags(&mut self, result: u32) {
        self.flags.zero = result == 0;
        self.flags.negative = (result as i32) < 0;
        // Carry and overflow are set by arithmetic operations
    }

    /// Set carry and overflow flags for addition
    pub fn set_add_flags(&mut self, a: u32, b: u32, result: u32) {
        // Carry: unsigned overflow
        self.flags.carry = result < a;

        // Overflow: signed overflow
        let a_sign = (a as i32) < 0;
        let b_sign = (b as i32) < 0;
        let r_sign = (result as i32) < 0;
        self.flags.overflow = (a_sign == b_sign) && (a_sign != r_sign);
    }

    /// Set carry and overflow flags for subtraction
    pub fn set_sub_flags(&mut self, a: u32, b: u32, result: u32) {
        // Carry: unsigned underflow (borrow)
        self.flags.carry = a < b;

        // Overflow: signed overflow
        let a_sign = (a as i32) < 0;
        let b_sign = (b as i32) < 0;
        let r_sign = (result as i32) < 0;
        self.flags.overflow = (a_sign != b_sign) && (a_sign != r_sign);
    }

    // ===== Execution State =====

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Halt the CPU
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Resume CPU execution
    pub fn resume(&mut self) {
        self.halted = false;
    }

    /// Get cycle count
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Increment cycle counter
    pub fn tick(&mut self) {
        self.cycle_count += 1;
    }

    /// Increment instruction counter
    pub fn count_instruction(&mut self) {
        self.instruction_count += 1;
    }

    // ===== Debugging =====

    /// Get a reference to all registers (for debugging/display)
    pub fn registers(&self) -> &[u32; NUM_REGISTERS] {
        &self.registers
    }

    /// Get a reference to memory (for debugging/display)
    pub fn memory(&self) -> &[u8; MEMORY_SIZE] {
        &self.memory
    }

    /// Get a slice of memory for a specific range
    pub fn memory_slice(&self, start: u32, len: usize) -> Result<&[u8], CpuError> {
        let start = start as usize;
        if start + len > MEMORY_SIZE {
            return Err(CpuError::MemoryOutOfBounds(start as u32, len));
        }
        Ok(&self.memory[start..start + len])
    }
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPU State:")?;
        writeln!(f, "  PC:     0x{:04X}", self.pc)?;
        writeln!(f, "  Flags:  {}", self.flags)?;
        writeln!(f, "  Cycles: {}", self.cycle_count)?;
        writeln!(f, "  Instr:  {}", self.instruction_count)?;
        writeln!(f, "  Halted: {}", self.halted)?;
        writeln!(f, "  Registers:")?;
        for i in 0..NUM_REGISTERS {
            let name = match i {
                0 => "zero",
                1 => "ra  ",
                2 => "sp  ",
                3 => "gp  ",
                4 => "tp  ",
                5 => "t0  ",
                6 => "t1  ",
                7 => "t2  ",
                8 => "s0  ",
                9 => "s1  ",
                10 => "a0  ",
                11 => "a1  ",
                12 => "a2  ",
                13 => "a3  ",
                14 => "a4  ",
                15 => "a5  ",
                16 => "a6  ",
                17 => "a7  ",
                18 => "s2  ",
                19 => "s3  ",
                20 => "s4  ",
                21 => "s5  ",
                22 => "s6  ",
                23 => "s7  ",
                24 => "s8  ",
                25 => "s9  ",
                26 => "s10 ",
                27 => "s11 ",
                28 => "t3  ",
                29 => "t4  ",
                30 => "t5  ",
                31 => "t6  ",
                _ => "    ",
            };
            writeln!(
                f,
                "    x{} ({}): 0x{:08X} ({})",
                i, name, self.registers[i], self.registers[i] as i32
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_creation() {
        let cpu = CpuState::new();
        assert_eq!(cpu.pc(), CODE_START);
        assert_eq!(cpu.stack_pointer(), INITIAL_SP);
        assert!(!cpu.is_halted());
        assert_eq!(cpu.cycle_count(), 0);
    }

    #[test]
    fn test_register_operations() {
        let mut cpu = CpuState::new();

        // x0 is always zero
        assert_eq!(cpu.read_register(0).unwrap(), 0);
        cpu.write_register(0, 42).unwrap();
        assert_eq!(cpu.read_register(0).unwrap(), 0);

        // Other registers work normally
        cpu.write_register(1, 100).unwrap();
        assert_eq!(cpu.read_register(1).unwrap(), 100);

        // Invalid register
        assert!(cpu.read_register(32).is_err());
        assert!(cpu.write_register(33, 0).is_err());
    }

    #[test]
    fn test_memory_operations() {
        let mut cpu = CpuState::new();

        // Byte operations
        cpu.write_byte(0x100, 0x42).unwrap();
        assert_eq!(cpu.read_byte(0x100).unwrap(), 0x42);

        // Word operations (aligned)
        cpu.write_word(0x100, 0xDEADBEEF).unwrap();
        assert_eq!(cpu.read_word(0x100).unwrap(), 0xDEADBEEF);

        // Unaligned access
        assert!(cpu.write_word(0x101, 0).is_err());
        assert!(cpu.read_word(0x102).is_err());

        // Out of bounds
        assert!(cpu.write_byte(MEMORY_SIZE as u32, 0).is_err());
        assert!(cpu.read_word(MEMORY_SIZE as u32 - 2).is_err());
    }

    #[test]
    fn test_flags() {
        let mut cpu = CpuState::new();

        // Zero result
        cpu.update_flags(0);
        assert!(cpu.flags().zero);
        assert!(!cpu.flags().negative);

        // Negative result
        cpu.update_flags(0xFFFFFFFF);
        assert!(!cpu.flags().zero);
        assert!(cpu.flags().negative);

        // Addition flags
        cpu.set_add_flags(0xFFFFFFFF, 1, 0);
        assert!(cpu.flags().carry); // Unsigned overflow

        // Signed overflow: MAX_POSITIVE + 1 = MIN_NEGATIVE
        cpu.set_add_flags(0x7FFFFFFF, 1, 0x80000000);
        assert!(cpu.flags().overflow);
    }

    #[test]
    fn test_pc_operations() {
        let mut cpu = CpuState::new();

        assert_eq!(cpu.pc(), CODE_START);

        cpu.set_pc(0x100).unwrap();
        assert_eq!(cpu.pc(), 0x100);

        cpu.increment_pc().unwrap();
        assert_eq!(cpu.pc(), 0x104);

        // Out of bounds
        assert!(cpu.set_pc(MEMORY_SIZE as u32).is_err());
    }

    #[test]
    fn test_halt() {
        let mut cpu = CpuState::new();

        assert!(!cpu.is_halted());
        cpu.halt();
        assert!(cpu.is_halted());
        cpu.resume();
        assert!(!cpu.is_halted());
    }

    #[test]
    fn test_reset() {
        let mut cpu = CpuState::new();

        cpu.write_register(1, 42).unwrap();
        cpu.set_pc(0x100).unwrap();
        cpu.halt();
        cpu.tick();
        cpu.count_instruction();

        cpu.reset();

        assert_eq!(cpu.read_register(1).unwrap(), 0);
        assert_eq!(cpu.pc(), CODE_START);
        assert!(!cpu.is_halted());
        assert_eq!(cpu.cycle_count(), 0);
        assert_eq!(cpu.instruction_count(), 0);
    }

    #[test]
    fn test_load_program() {
        let mut cpu = CpuState::new();

        let program = vec![0x13, 0x01, 0x50, 0x00]; // ADDI x1, x0, 5
        cpu.load_program(CODE_START, &program).unwrap();

        assert_eq!(cpu.read_byte(CODE_START).unwrap(), 0x13);
        assert_eq!(cpu.read_word(CODE_START).unwrap(), 0x00500113);

        // Out of bounds
        let too_large = vec![0; MEMORY_SIZE + 1];
        assert!(cpu.load_program(0, &too_large).is_err());
    }
}
