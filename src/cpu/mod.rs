//! CPU simulator module
//!
//! Contains a simplified RISC-V CPU implementation for educational purposes.

pub mod executor;
pub mod instruction;
pub mod state;

// Re-export main types
pub use executor::execute;
pub use instruction::Instruction;
pub use state::{CpuError, CpuState, StatusFlags};
