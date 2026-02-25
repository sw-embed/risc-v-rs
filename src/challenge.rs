//! Challenge system for the assembly game
//!
//! Defines puzzles, test cases, and validation logic.

use crate::cpu::{CpuState, Instruction, execute};
use serde::{Deserialize, Serialize};

/// Difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// A single test case for a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Name of this test case
    pub name: String,

    /// Initial register values (register_index, value)
    pub initial_registers: Vec<(u8, u32)>,

    /// Expected register values after execution (register_index, value)
    pub expected_registers: Vec<(u8, u32)>,

    /// Expected memory values (address, value)
    #[serde(default)]
    pub expected_memory: Vec<(u32, u8)>,
}

impl TestCase {
    /// Check if the CPU state matches expected values
    pub fn validate(&self, cpu: &CpuState) -> Result<(), String> {
        // Check registers
        for (reg, expected) in &self.expected_registers {
            let actual = cpu
                .read_register(*reg)
                .map_err(|e| format!("Invalid register: {}", e))?;

            if actual != *expected {
                return Err(format!(
                    "Register x{} mismatch: expected 0x{:08X}, got 0x{:08X}",
                    reg, expected, actual
                ));
            }
        }

        // Check memory
        for (addr, expected) in &self.expected_memory {
            let actual = cpu
                .read_byte(*addr)
                .map_err(|e| format!("Invalid memory address: {}", e))?;

            if actual != *expected {
                return Err(format!(
                    "Memory[0x{:04X}] mismatch: expected 0x{:02X}, got 0x{:02X}",
                    addr, expected, actual
                ));
            }
        }

        Ok(())
    }
}

/// A challenge/puzzle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Unique challenge ID
    pub id: u32,

    /// Challenge title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Difficulty level
    pub difficulty: Difficulty,

    /// Test cases to validate the solution
    pub test_cases: Vec<TestCase>,

    /// Maximum allowed cycles (None = no limit)
    pub max_cycles: Option<u64>,

    /// Maximum allowed instructions (None = no limit)
    pub max_instructions: Option<u64>,

    /// Hints for the player
    #[serde(default)]
    pub hints: Vec<String>,

    /// Learning objectives
    #[serde(default)]
    pub learning_objectives: Vec<String>,
}

impl Challenge {
    /// Run all test cases against a program
    pub fn validate_solution(&self, program: &[Instruction]) -> Result<ValidationResult, String> {
        let mut results = Vec::new();

        for (i, test_case) in self.test_cases.iter().enumerate() {
            let mut cpu = CpuState::new();

            // Set up initial register state
            for (reg, value) in &test_case.initial_registers {
                cpu.write_register(*reg, *value)
                    .map_err(|e| format!("Test {}: Failed to set register: {}", i + 1, e))?;
            }

            // Execute program
            let max_iterations = 10000;

            for (instruction_count, inst) in program.iter().enumerate() {
                if instruction_count >= max_iterations {
                    return Err(format!(
                        "Test {}: Execution timeout (infinite loop?)",
                        i + 1
                    ));
                }

                match execute(&mut cpu, *inst) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break; // HALT
                        }
                    }
                    Err(e) => {
                        return Err(format!("Test {}: Execution error: {}", i + 1, e));
                    }
                }

                // Check constraints
                if let Some(max_cycles) = self.max_cycles
                    && cpu.cycle_count() > max_cycles
                {
                    return Err(format!(
                        "Test {}: Exceeded cycle limit ({} > {})",
                        i + 1,
                        cpu.cycle_count(),
                        max_cycles
                    ));
                }

                if let Some(max_insts) = self.max_instructions
                    && cpu.instruction_count() > max_insts
                {
                    return Err(format!(
                        "Test {}: Exceeded instruction limit ({} > {})",
                        i + 1,
                        cpu.instruction_count(),
                        max_insts
                    ));
                }
            }

            // Validate test case
            match test_case.validate(&cpu) {
                Ok(()) => results.push(TestResult {
                    test_name: test_case.name.clone(),
                    passed: true,
                    error: None,
                    cycles: cpu.cycle_count(),
                    instructions: cpu.instruction_count(),
                }),
                Err(e) => results.push(TestResult {
                    test_name: test_case.name.clone(),
                    passed: false,
                    error: Some(e),
                    cycles: cpu.cycle_count(),
                    instructions: cpu.instruction_count(),
                }),
            }
        }

        let all_passed = results.iter().all(|r| r.passed);

        Ok(ValidationResult {
            challenge_id: self.id,
            passed: all_passed,
            test_results: results,
        })
    }
}

/// Result of validating a single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub cycles: u64,
    pub instructions: u64,
}

/// Result of validating an entire challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub challenge_id: u32,
    pub passed: bool,
    pub test_results: Vec<TestResult>,
}

/// Get all available challenges
pub fn get_all_challenges() -> Vec<Challenge> {
    vec![
        challenge_1_load_and_halt(),
        challenge_2_simple_addition(),
        challenge_3_register_transfer(),
    ]
}

/// Challenge 1: Load and Halt
fn challenge_1_load_and_halt() -> Challenge {
    Challenge {
        id: 1,
        title: "Challenge 1: Load and Halt".to_string(),
        description: "Load the number 42 into register x1, then halt the CPU.\n\n\
                     Use the ADDI instruction to load an immediate value.\n\
                     Remember to end with HALT!"
            .to_string(),
        difficulty: Difficulty::Beginner,
        test_cases: vec![TestCase {
            name: "x1 should contain 42".to_string(),
            initial_registers: vec![],
            expected_registers: vec![(1, 42)],
            expected_memory: vec![],
        }],
        max_cycles: Some(100),
        max_instructions: Some(10),
        hints: vec![
            "ADDI adds an immediate value to a register".to_string(),
            "x0 is always zero, so ADDI x1, x0, 42 sets x1 to 42".to_string(),
            "Don't forget the HALT instruction!".to_string(),
        ],
        learning_objectives: vec![
            "Understand immediate instructions".to_string(),
            "Learn about the HALT instruction".to_string(),
            "Practice basic instruction syntax".to_string(),
        ],
    }
}

/// Challenge 2: Simple Addition
fn challenge_2_simple_addition() -> Challenge {
    Challenge {
        id: 2,
        title: "Challenge 2: Simple Addition".to_string(),
        description: "Add 5 + 3 and store the result in register x1.\n\n\
                     You can use ADDI to load values and ADD to add them."
            .to_string(),
        difficulty: Difficulty::Beginner,
        test_cases: vec![TestCase {
            name: "x1 should contain 8 (5 + 3)".to_string(),
            initial_registers: vec![],
            expected_registers: vec![(1, 8)],
            expected_memory: vec![],
        }],
        max_cycles: Some(100),
        max_instructions: Some(20),
        hints: vec![
            "Load 5 into one register, 3 into another".to_string(),
            "Use ADD to combine them".to_string(),
            "You could also use ADDI x1, x0, 5 followed by ADDI x1, x1, 3".to_string(),
        ],
        learning_objectives: vec![
            "Understand arithmetic operations".to_string(),
            "Learn to chain instructions".to_string(),
            "Practice using multiple registers".to_string(),
        ],
    }
}

/// Challenge 3: Register Transfer
fn challenge_3_register_transfer() -> Challenge {
    Challenge {
        id: 3,
        title: "Challenge 3: Register Transfer".to_string(),
        description:
            "Copy the value from register x1 to register x2 WITHOUT using an immediate value.\n\n\
                     x1 starts with the value 100. Your task is to make x2 also equal 100."
                .to_string(),
        difficulty: Difficulty::Beginner,
        test_cases: vec![TestCase {
            name: "x2 should equal x1 (both 100)".to_string(),
            initial_registers: vec![(1, 100)],
            expected_registers: vec![(1, 100), (2, 100)],
            expected_memory: vec![],
        }],
        max_cycles: Some(100),
        max_instructions: Some(10),
        hints: vec![
            "Remember that x0 is always zero".to_string(),
            "Adding x1 + 0 gives you x1".to_string(),
            "Try: ADD x2, x1, x0".to_string(),
        ],
        learning_objectives: vec![
            "Understand the zero register (x0)".to_string(),
            "Learn register-to-register operations".to_string(),
            "Practice using ADD for non-arithmetic purposes".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_1_valid_solution() {
        let challenge = challenge_1_load_and_halt();

        let program = vec![
            Instruction::Addi {
                rd: 1,
                rs1: 0,
                imm: 42,
            },
            Instruction::Halt,
        ];

        let result = challenge.validate_solution(&program).unwrap();
        assert!(result.passed);
        assert_eq!(result.test_results.len(), 1);
        assert!(result.test_results[0].passed);
    }

    #[test]
    fn test_challenge_1_wrong_value() {
        let challenge = challenge_1_load_and_halt();

        let program = vec![
            Instruction::Addi {
                rd: 1,
                rs1: 0,
                imm: 99, // Wrong value!
            },
            Instruction::Halt,
        ];

        let result = challenge.validate_solution(&program).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_challenge_2_valid_solution() {
        let challenge = challenge_2_simple_addition();

        let program = vec![
            Instruction::Addi {
                rd: 2,
                rs1: 0,
                imm: 5,
            },
            Instruction::Addi {
                rd: 3,
                rs1: 0,
                imm: 3,
            },
            Instruction::Add {
                rd: 1,
                rs1: 2,
                rs2: 3,
            },
            Instruction::Halt,
        ];

        let result = challenge.validate_solution(&program).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_challenge_3_valid_solution() {
        let challenge = challenge_3_register_transfer();

        let program = vec![
            Instruction::Add {
                rd: 2,
                rs1: 1,
                rs2: 0,
            }, // x2 = x1 + x0
            Instruction::Halt,
        ];

        let result = challenge.validate_solution(&program).unwrap();
        assert!(result.passed);
        assert_eq!(
            result.test_results[0].test_name,
            "x2 should equal x1 (both 100)"
        );
    }
}
