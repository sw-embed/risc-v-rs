//! RISC-V Assembly Game - Educational Programming Challenge
//!
//! A browser-based game that teaches assembly programming through interactive puzzles.
//!
//! This example contains game-specific components (CPU simulator, challenge system, UI)
//! that are unlikely to be reused by other types of games in the library.

pub mod assembler;
pub mod challenge;
pub mod cpu;

// Yew app (only for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod app;

// WASM bindings (only for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export main types for convenience
pub use assembler::{AssemblyError, AssemblyOutput, assemble};
pub use challenge::{Challenge, Difficulty, TestCase, ValidationResult};
pub use cpu::{CpuError, CpuState, Instruction, execute};
