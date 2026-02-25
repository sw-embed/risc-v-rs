# RISC-V RV32I Assembly Emulator

An interactive browser-based educational game for learning the RISC-V RV32I instruction set architecture. RISC-V is an open-source instruction set architecture (ISA) that has become increasingly popular in academia and industry.

## Live Demo

**[Try it online](https://sw-embed.github.io/risc-v-rs/)**

![RISC-V RV32I Emulator Screenshot](images/risc-v-interface.png?ts=1772049940000)

## Features

- **RISC-V RV32I CPU emulation** - Base 32-bit integer instruction set
- **32 General Purpose Registers** - x0-x31 (x0 is hardwired to zero)
- **Standard instruction formats** - R, I, S, B, U, J types
- **Complete instruction subset** - ADD, SUB, AND, OR, XOR, SLL, SRL, SRA, ADDI, LW, SW, BEQ, BNE, JAL, JALR, LUI, AUIPC
- **Interactive examples** covering arithmetic, memory access, branching, and function calls
- **Progressive challenges** with validation
- **Real-time visualization** of CPU state, registers, and memory

## Documentation

- [Porting Guide](docs/porting.md) - How this project was extracted from game-lib

## Architecture

The RISC-V RV32I emulator implements the base 32-bit integer instruction set:

- **x0-x31** - 32 General Purpose Registers (32-bit each)
- **x0** - Hardwired to zero (reads always return 0, writes are ignored)
- **PC** - Program Counter
- **Instruction Formats**:
  - **R-type** - Register-register operations (ADD, SUB, AND, OR, XOR, SLL, SRL, SRA)
  - **I-type** - Immediate operations (ADDI, LW, JALR)
  - **S-type** - Store operations (SW)
  - **B-type** - Branch operations (BEQ, BNE, BLT, BGE)
  - **U-type** - Upper immediate (LUI, AUIPC)
  - **J-type** - Jump (JAL)

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (with wasm32-unknown-unknown target)
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`

### Development

```bash
# Run development server with hot reload
trunk serve

# Build for production
trunk build --release
```

The production build outputs to `./pages/`.

### Deploying to GitHub Pages

1. Build locally:
   ```bash
   trunk build --release
   ```

2. Update gh-pages branch:
   ```bash
   git checkout gh-pages
   rm -rf *.js *.wasm *.css index.html
   cp -r pages/* .
   git add .
   git commit -m "Deploy"
   git push
   git checkout main
   ```

## Project Structure

```
risc-v-rs/
├── src/                    # Main application
│   ├── app.rs             # Yew application component
│   ├── assembler.rs       # Assembly parser
│   ├── challenge.rs       # Challenge system
│   ├── cpu/               # CPU emulation
│   │   ├── executor.rs    # Instruction execution
│   │   ├── instruction.rs # Instruction definitions
│   │   └── state.rs       # CPU state management
│   ├── lib.rs             # Library root
│   └── wasm.rs            # WASM bindings
├── components/            # Shared Yew UI components
│   └── src/
│       ├── components/    # UI components (header, sidebar, etc.)
│       └── lib.rs
├── styles/                # CSS stylesheets
├── docs/                  # Documentation
├── images/                # Screenshots
├── index.html             # HTML entry point
├── Trunk.toml             # Trunk configuration
└── Cargo.toml             # Workspace configuration
```

## Instruction Set Summary

### R-Type (Register-Register)
| Instruction | Operation |
|-------------|-----------|
| ADD rd, rs1, rs2 | rd = rs1 + rs2 |
| SUB rd, rs1, rs2 | rd = rs1 - rs2 |
| AND rd, rs1, rs2 | rd = rs1 & rs2 |
| OR rd, rs1, rs2 | rd = rs1 \| rs2 |
| XOR rd, rs1, rs2 | rd = rs1 ^ rs2 |
| SLL rd, rs1, rs2 | rd = rs1 << rs2 |
| SRL rd, rs1, rs2 | rd = rs1 >> rs2 (logical) |
| SRA rd, rs1, rs2 | rd = rs1 >> rs2 (arithmetic) |
| SLT rd, rs1, rs2 | rd = (rs1 < rs2) ? 1 : 0 (signed) |
| SLTU rd, rs1, rs2 | rd = (rs1 < rs2) ? 1 : 0 (unsigned) |

### I-Type (Immediate)
| Instruction | Operation |
|-------------|-----------|
| ADDI rd, rs1, imm | rd = rs1 + imm |
| ANDI rd, rs1, imm | rd = rs1 & imm |
| ORI rd, rs1, imm | rd = rs1 \| imm |
| XORI rd, rs1, imm | rd = rs1 ^ imm |
| SLTI rd, rs1, imm | rd = (rs1 < imm) ? 1 : 0 |
| LW rd, imm(rs1) | rd = Memory[rs1 + imm] |
| JALR rd, imm(rs1) | rd = PC+4; PC = rs1 + imm |

### S-Type (Store)
| Instruction | Operation |
|-------------|-----------|
| SW rs2, imm(rs1) | Memory[rs1 + imm] = rs2 |

### B-Type (Branch)
| Instruction | Operation |
|-------------|-----------|
| BEQ rs1, rs2, offset | if (rs1 == rs2) PC += offset |
| BNE rs1, rs2, offset | if (rs1 != rs2) PC += offset |
| BLT rs1, rs2, offset | if (rs1 < rs2) PC += offset |
| BGE rs1, rs2, offset | if (rs1 >= rs2) PC += offset |

### U-Type (Upper Immediate)
| Instruction | Operation |
|-------------|-----------|
| LUI rd, imm | rd = imm << 12 |
| AUIPC rd, imm | rd = PC + (imm << 12) |

### J-Type (Jump)
| Instruction | Operation |
|-------------|-----------|
| JAL rd, offset | rd = PC+4; PC += offset |

### Pseudo-Instructions
| Instruction | Expansion |
|-------------|-----------|
| NOP | ADDI x0, x0, 0 |
| MV rd, rs | ADDI rd, rs, 0 |
| LI rd, imm | Various (depends on immediate size) |
| HALT | Custom halt instruction (99) |

## References

### RISC-V Foundation

- **[RISC-V Specifications](https://riscv.org/technical/specifications/)** - Official RISC-V ISA specifications
- **[RISC-V Unprivileged ISA](https://github.com/riscv/riscv-isa-manual)** - The official ISA manual

### Educational Resources

- **[RISC-V - Wikipedia](https://en.wikipedia.org/wiki/RISC-V)** - Overview and history
- **[Computer Organization and Design RISC-V Edition](https://www.elsevier.com/books/computer-organization-and-design-risc-v-edition/patterson/978-0-12-812275-4)** - Patterson & Hennessy textbook

### Historical Context

RISC-V originated at UC Berkeley in 2010:
- Open-source ISA with no licensing fees
- Clean-slate design avoiding legacy baggage
- Modular design with base ISA and optional extensions
- Growing adoption in academia, embedded systems, and custom silicon
- RV32I is the base 32-bit integer instruction set (this emulator)
- Extensions include M (multiply), A (atomic), F/D (floating-point), C (compressed)

## License

MIT
