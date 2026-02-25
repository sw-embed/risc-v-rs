//! WASM bindings for the CPU simulator
//!
//! This module provides JavaScript-accessible interfaces to the CPU.

use crate::challenge::get_all_challenges;
use crate::cpu::{CpuState, Instruction, execute};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// WASM-accessible CPU wrapper
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmCpu {
    cpu: CpuState,
    program: Vec<Instruction>,
}

#[wasm_bindgen]
impl WasmCpu {
    /// Create a new CPU
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();

        Self {
            cpu: CpuState::new(),
            program: Vec::new(),
        }
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    /// Assemble source code and load into program
    pub fn assemble(&mut self, source: &str) -> Result<JsValue, JsValue> {
        let output =
            crate::assembler::assemble(source).map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.program = output.instructions.clone();

        console::log_1(&JsValue::from_str(&format!(
            "Loaded {} instructions",
            self.program.len()
        )));

        // Return assembly output as JSON
        serde_wasm_bindgen::to_value(&output).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Execute one instruction
    pub fn step(&mut self) -> Result<bool, JsValue> {
        let pc = self.cpu.pc() as usize;
        let inst_index = pc / 4;

        if inst_index >= self.program.len() {
            return Err(JsValue::from_str("PC out of program bounds"));
        }

        let inst = self.program[inst_index];

        execute(&mut self.cpu, inst)
            .map_err(|e| JsValue::from_str(&format!("Execution error: {}", e)))
    }

    /// Run until halt or error
    pub fn run(&mut self) -> Result<(), JsValue> {
        let max_iterations = 10000;
        let mut iterations = 0;

        loop {
            match self.step() {
                Ok(should_continue) => {
                    if !should_continue {
                        break; // Halted
                    }
                }
                Err(e) => return Err(e),
            }

            iterations += 1;
            if iterations >= max_iterations {
                return Err(JsValue::from_str("Execution timeout (infinite loop?)"));
            }
        }

        Ok(())
    }

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }

    /// Get program counter
    pub fn pc(&self) -> u32 {
        self.cpu.pc()
    }

    /// Get cycle count
    pub fn cycle_count(&self) -> u32 {
        self.cpu.cycle_count() as u32
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> u32 {
        self.cpu.instruction_count() as u32
    }

    /// Read a register value
    pub fn read_register(&self, reg: u8) -> Result<u32, JsValue> {
        self.cpu
            .read_register(reg)
            .map_err(|e| JsValue::from_str(&format!("Register error: {}", e)))
    }

    /// Get all register values as an array
    pub fn get_registers(&self) -> Vec<u32> {
        self.cpu.registers().to_vec()
    }

    /// Get status flags as a string
    pub fn get_flags(&self) -> String {
        self.cpu.flags().to_string()
    }

    /// Read memory byte
    pub fn read_memory(&self, addr: u32) -> Result<u8, JsValue> {
        self.cpu
            .read_byte(addr)
            .map_err(|e| JsValue::from_str(&format!("Memory error: {}", e)))
    }

    /// Get memory slice as bytes
    pub fn get_memory_slice(&self, start: u32, len: u32) -> Result<Vec<u8>, JsValue> {
        self.cpu
            .memory_slice(start, len as usize)
            .map(|slice| slice.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Memory error: {}", e)))
    }
}

// ===== Challenge System =====

/// Get number of available challenges
#[wasm_bindgen]
pub fn get_challenge_count() -> u32 {
    get_all_challenges().len() as u32
}

/// Get challenge data as JSON string
#[wasm_bindgen]
pub fn get_challenge_json(id: u32) -> Result<String, JsValue> {
    let challenges = get_all_challenges();

    let challenge = challenges
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| JsValue::from_str(&format!("Challenge {} not found", id)))?;

    serde_json::to_string(&challenge)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validate a solution for a challenge
/// Returns JSON string with validation results
#[wasm_bindgen]
pub fn validate_challenge(challenge_id: u32, source: &str) -> Result<String, JsValue> {
    // Find the challenge
    let challenges = get_all_challenges();
    let challenge = challenges
        .into_iter()
        .find(|c| c.id == challenge_id)
        .ok_or_else(|| JsValue::from_str(&format!("Challenge {} not found", challenge_id)))?;

    // Assemble the source code
    let output = crate::assembler::assemble(source)
        .map_err(|e| JsValue::from_str(&format!("Assembly error: {}", e)))?;

    // Validate
    let result = challenge
        .validate_solution(&output.instructions)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    // Return as JSON
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Initialize the WASM module and mount Yew app
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in browser console
    console_error_panic_hook::set_once();

    // Mount the Yew app
    yew::Renderer::<crate::app::App>::new().render();
}
