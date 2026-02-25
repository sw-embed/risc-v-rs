//! Yew application for RV32 Assembly Game

use components::{
    Header, LegendItem, MemoryViewer, Modal, ProgramArea, Register, RegisterPanel, Sidebar,
    SidebarButton,
};
use yew::prelude::*;

use crate::wasm::WasmCpu;

#[function_component(App)]
pub fn app() -> Html {
    // State management
    let cpu = use_state(|| WasmCpu::new());
    let program_code = use_state(|| String::from(EXAMPLE_PROGRAM));
    let assembly_output = use_state(|| None::<Html>);
    let assembly_lines = use_state(|| Vec::<String>::new());
    let last_registers = use_state(|| vec![0u32; 8]);
    let challenge_mode = use_state(|| false);
    let current_challenge_id = use_state(|| None::<u32>);
    let challenge_result = use_state(|| None::<Result<String, String>>);

    // Modal states
    let tutorial_open = use_state(|| false);
    let examples_open = use_state(|| false);
    let challenges_open = use_state(|| false);
    let isa_ref_open = use_state(|| false);
    let help_open = use_state(|| false);

    // Callbacks for modals
    let close_tutorial = {
        let tutorial_open = tutorial_open.clone();
        Callback::from(move |_| tutorial_open.set(false))
    };
    let close_examples = {
        let examples_open = examples_open.clone();
        Callback::from(move |_| examples_open.set(false))
    };
    let close_challenges = {
        let challenges_open = challenges_open.clone();
        Callback::from(move |_| challenges_open.set(false))
    };
    let close_isa_ref = {
        let isa_ref_open = isa_ref_open.clone();
        Callback::from(move |_| isa_ref_open.set(false))
    };
    let close_help = {
        let help_open = help_open.clone();
        Callback::from(move |_| help_open.set(false))
    };

    // Example programs
    let examples = vec![
        (
            "Example 1: Add Two Numbers",
            "; Load 5 into x1\nADDI x1, x0, 5\n\n; Load 10 into x2\nADDI x2, x0, 10\n\n; Add x1 + x2 -> x3\nADD x3, x1, x2\n\n; Halt\nHALT",
        ),
        (
            "Example 2: Subtraction",
            "; Load 20 into x1\nADDI x1, x0, 20\n\n; Load 7 into x2\nADDI x2, x0, 7\n\n; Subtract: x3 = x1 - x2\nSUB x3, x1, x2\n\n; Halt\nHALT",
        ),
        (
            "Example 3: Chained Operations",
            "; Load 3 into x1\nADDI x1, x0, 3\n\n; Add 5: x1 = x1 + 5\nADDI x1, x1, 5\n\n; Add 2: x1 = x1 + 2\nADDI x1, x1, 2\n\n; Result: x1 = 10\nHALT",
        ),
        (
            "Example 4: Loop with Branch",
            "; Initialize counter\nADDI x1, x0, 0\n\nLOOP:\n; Increment counter\nADDI x1, x1, 1\n\n; Check if x1 < 5\nADDI x2, x0, 5\nBLT x1, x2, LOOP\n\n; Done\nHALT",
        ),
    ];

    // Sidebar buttons with inline callbacks
    let sidebar_buttons = vec![
        SidebarButton {
            emoji: "📚".to_string(),
            label: "Tutorial".to_string(),
            onclick: {
                let tutorial_open = tutorial_open.clone();
                Callback::from(move |_| tutorial_open.set(true))
            },
            title: Some("Learn RISC-V basics".to_string()),
        },
        SidebarButton {
            emoji: "📝".to_string(),
            label: "Examples".to_string(),
            onclick: {
                let examples_open = examples_open.clone();
                Callback::from(move |_| examples_open.set(true))
            },
            title: Some("Load example programs".to_string()),
        },
        SidebarButton {
            emoji: "🎯".to_string(),
            label: "Challenges".to_string(),
            onclick: {
                let challenges_open = challenges_open.clone();
                Callback::from(move |_| challenges_open.set(true))
            },
            title: Some("Test your skills".to_string()),
        },
        SidebarButton {
            emoji: "📖".to_string(),
            label: "ISA Ref".to_string(),
            onclick: {
                let isa_ref_open = isa_ref_open.clone();
                Callback::from(move |_| isa_ref_open.set(true))
            },
            title: Some("Instruction reference".to_string()),
        },
        SidebarButton {
            emoji: "❓".to_string(),
            label: "Help".to_string(),
            onclick: {
                let help_open = help_open.clone();
                Callback::from(move |_| help_open.set(true))
            },
            title: Some("Usage help".to_string()),
        },
    ];

    // CPU operation callbacks
    let on_assemble = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();
        let program_code = program_code.clone();

        Callback::from(move |code: String| {
            program_code.set(code.clone());

            // Assemble the source code
            let mut new_cpu = (*cpu).clone();
            match new_cpu.assemble(&code) {
                Ok(output) => {
                    cpu.set(new_cpu);

                    // Store disassembly lines for highlighting
                    if let Ok(asm_output) =
                        serde_wasm_bindgen::from_value::<crate::assembler::AssemblyOutput>(output)
                    {
                        assembly_lines.set(asm_output.disassembly.clone());

                        assembly_output.set(Some(html! {
                            <div class="success-text">
                                {"✓ Program assembled successfully"}
                            </div>
                        }));
                    } else {
                        assembly_lines.set(Vec::new());
                        assembly_output.set(Some(html! {
                            <div class="success-text">
                                {"✓ Program assembled successfully"}
                            </div>
                        }));
                    }
                }
                Err(e) => {
                    assembly_lines.set(Vec::new());
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Assembly error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_step = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let last_registers = last_registers.clone();

        Callback::from(move |()| {
            let mut new_cpu = (*cpu).clone();

            // Save current state for change tracking
            last_registers.set(new_cpu.get_registers().to_vec());

            match new_cpu.step() {
                Ok(_) => {
                    cpu.set(new_cpu);
                    // Don't set assembly_output - let the highlighting show in the main display
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_run = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();

        Callback::from(move |()| {
            let mut new_cpu = (*cpu).clone();
            match new_cpu.run() {
                Ok(()) => {
                    cpu.set(new_cpu);
                    assembly_output.set(Some(html! {
                        <div class="success-text">
                            {"✓ Program completed"}
                        </div>
                    }));
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_reset = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();

        Callback::from(move |()| {
            // Full reset - create new CPU with cleared memory
            cpu.set(WasmCpu::new());
            assembly_lines.set(Vec::new());
            assembly_output.set(None);
        })
    };

    // Register panel data
    let registers = {
        let regs = (*cpu).get_registers();
        let last_regs = &*last_registers;
        let mut reg_list = Vec::new();
        for i in 0..8 {
            let changed = regs[i] != last_regs[i];
            reg_list.push(Register {
                name: format!("x{}", i),
                value: format!("0x{:08X} ({})", regs[i], regs[i] as i32),
                changed,
            });
        }
        reg_list
    };

    let legend_items = vec![
        LegendItem {
            label: "zero".to_string(),
            value: "Always 0".to_string(),
            changed: false,
        },
        LegendItem {
            label: "ra".to_string(),
            value: "Return Address".to_string(),
            changed: false,
        },
        LegendItem {
            label: "sp".to_string(),
            value: "Stack Pointer".to_string(),
            changed: false,
        },
        LegendItem {
            label: "gp".to_string(),
            value: "Global Pointer".to_string(),
            changed: false,
        },
        LegendItem {
            label: "tp".to_string(),
            value: "Thread Pointer".to_string(),
            changed: false,
        },
        LegendItem {
            label: "t0-t2".to_string(),
            value: "Temporaries".to_string(),
            changed: false,
        },
    ];

    // Memory data
    let memory = match (*cpu).get_memory_slice(0, 128) {
        Ok(mem) => mem,
        Err(_) => vec![0u8; 128],
    };

    let pc = (*cpu).pc() as u16;

    html! {
        <div class="container">
            <Header title="RISC-V RV32 Assembly Game" />

            <Sidebar buttons={sidebar_buttons} />

            <div class="main-content">
                <ProgramArea
                    on_assemble={on_assemble}
                    on_step={on_step}
                    on_run={on_run}
                    on_reset={on_reset}
                    assembly_output={
                        if !assembly_lines.is_empty() {
                            // Show highlighted assembly lines
                            let pc = (*cpu).pc();
                            Some(html! {
                                <div>
                                    {for assembly_lines.iter().map(|line| {
                                        // Parse address from "ADDRESS: INSTRUCTION" format
                                        let addr_str = line.split(':').next().unwrap_or("");
                                        let is_current = if let Ok(addr) = u32::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                                            addr == pc
                                        } else {
                                            false
                                        };

                                        let class = if is_current {
                                            "assembly-line current"
                                        } else {
                                            "assembly-line"
                                        };

                                        html! {
                                            <div class={class}>{line}</div>
                                        }
                                    })}
                                </div>
                            })
                        } else {
                            // Show success/error messages
                            (*assembly_output).clone()
                        }
                    }
                    initial_code={Some((*program_code).clone())}
                    step_enabled={!(*cpu).is_halted()}
                    run_enabled={!(*cpu).is_halted()}
                />

                <div class="right-panels">
                    <div class="registers-panel">
                        <RegisterPanel
                            registers={registers}
                            legend_items={legend_items}
                        />

                        // CPU Status
                        <div class="cpu-status">
                            <div class="status-item">
                                <span class="status-label">{"Cycles:"}</span>
                                <span class="status-value">{(*cpu).cycle_count()}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Instructions:"}</span>
                                <span class="status-value">{(*cpu).instruction_count()}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Status:"}</span>
                                <span class="status-value">
                                    {if (*cpu).is_halted() { "HALTED" } else { "RUNNING" }}
                                </span>
                            </div>
                        </div>
                    </div>

                    <MemoryViewer
                        memory={memory}
                        pc={pc}
                        title={Some("Memory (First 128 Bytes)".to_string())}
                        bytes_per_row={16}
                        bytes_to_show={128}
                    />
                </div>
            </div>

            // Challenge Mode Banner
            if *challenge_mode {
                if let Some(challenge_id) = *current_challenge_id {
                    <div class="challenge-banner">
                        <span class="challenge-indicator">{"⚡"}</span>
                        <span class="challenge-text">
                            {format!("Challenge Mode - Challenge {}", challenge_id)}
                        </span>
                        <button
                            class="check-button"
                            onclick={
                                let challenge_result = challenge_result.clone();
                                let program_code = program_code.clone();
                                Callback::from(move |_| {
                                    // Validate assembly source
                                    match crate::wasm::validate_challenge(challenge_id, &(*program_code)) {
                                                Ok(json_result) => {
                                                    match serde_json::from_str::<crate::challenge::ValidationResult>(&json_result) {
                                                        Ok(validation) => {
                                                            if validation.passed {
                                                                let mut message = format!("✅ Challenge {} PASSED!\n\n", validation.challenge_id);
                                                                for test in &validation.test_results {
                                                                    message.push_str(&format!("✓ {}\n", test.test_name));
                                                                    message.push_str(&format!("  Cycles: {}, Instructions: {}\n", test.cycles, test.instructions));
                                                                }
                                                                challenge_result.set(Some(Ok(message)));
                                                            } else {
                                                                let mut message = format!("❌ Challenge {} did not pass.\n\n", validation.challenge_id);
                                                                for test in &validation.test_results {
                                                                    if test.passed {
                                                                        message.push_str(&format!("✓ {}\n", test.test_name));
                                                                    } else {
                                                                        message.push_str(&format!("✗ {}\n", test.test_name));
                                                                        if let Some(error) = &test.error {
                                                                            message.push_str(&format!("  Error: {}\n", error));
                                                                        }
                                                                    }
                                                                }
                                                                challenge_result.set(Some(Err(message)));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            challenge_result.set(Some(Err(format!("Failed to parse validation result: {}", e))));
                                                        }
                                                    }
                                                }
                                        Err(e) => {
                                            challenge_result.set(Some(Err(format!("Validation error: {:?}", e))));
                                        }
                                    }
                                })
                            }
                        >
                            {"Check Solution"}
                        </button>
                        <button
                            class="exit-button"
                            onclick={
                                let challenge_mode = challenge_mode.clone();
                                let current_challenge_id = current_challenge_id.clone();
                                let challenge_result = challenge_result.clone();
                                Callback::from(move |_| {
                                    challenge_mode.set(false);
                                    current_challenge_id.set(None);
                                    challenge_result.set(None);
                                })
                            }
                        >
                            {"Exit"}
                        </button>
                    </div>
                }
            }

            // Success/Error Banners
            {
                if let Some(result) = &*challenge_result {
                    match result {
                        Ok(message) => html! {
                            <div class="success-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        },
                        Err(message) => html! {
                            <div class="error-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        }
                    }
                } else {
                    html! {}
                }
            }

            // Modals
            <Modal id="tutorial" title="Tutorial" active={*tutorial_open} on_close={close_tutorial}>
                {html! { <div>{TUTORIAL_CONTENT}</div> }}
            </Modal>

            <Modal id="examples" title="Examples" active={*examples_open} on_close={close_examples}>
                <div class="examples-list">
                    {for examples.iter().enumerate().map(|(idx, (title, code))| {
                        let program_code = program_code.clone();
                        let examples_open = examples_open.clone();
                        let cpu = cpu.clone();
                        let assembly_output = assembly_output.clone();
                        let code = code.to_string();

                        let load_example = Callback::from(move |_: MouseEvent| {
                            // Reset CPU
                            cpu.set(WasmCpu::new());
                            assembly_output.set(None);

                            // Load new code
                            program_code.set(code.clone());
                            examples_open.set(false);
                        });

                        html! {
                            <div class="example-item" key={idx} onclick={load_example}>
                                <h4>{title}</h4>
                                <p>{"Click to load this example"}</p>
                            </div>
                        }
                    })}
                </div>
            </Modal>

            <Modal id="challenges" title="Challenges" active={*challenges_open} on_close={close_challenges}>
                {render_challenges_list(challenge_mode.clone(), current_challenge_id.clone(), program_code.clone(), challenges_open.clone())}
            </Modal>

            <Modal id="isaRef" title="ISA Reference" active={*isa_ref_open} on_close={close_isa_ref}>
                {html! { <div>{ISA_REF_CONTENT}</div> }}
            </Modal>

            <Modal id="help" title="Help" active={*help_open} on_close={close_help}>
                {html! { <div>{HELP_CONTENT}</div> }}
            </Modal>

            // GitHub Corner
            <a href="https://github.com/sw-embed/risc-v-rs" class="github-corner" aria-label="View source on GitHub" target="_blank">
                <svg width="80" height="80" viewBox="0 0 250 250" style="fill:#00d9ff; color:#1a1a2e; position: absolute; top: 0; border: 0; right: 0;" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"></path>
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor" style="transform-origin: 130px 106px;" class="octo-arm"></path>
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z" fill="currentColor" class="octo-body"></path>
                </svg>
            </a>

            // Footer
            <footer class="app-footer">
                <div class="footer-left">
                    <span>{"MIT License"}</span>
                    <span>{"© 2026 Michael A Wright"}</span>
                </div>
                <div class="footer-right">
                    <span>{format!("{} | {} | {}", env!("VERGEN_BUILD_HOST"), env!("VERGEN_GIT_SHA_SHORT"), env!("VERGEN_BUILD_TIMESTAMP"))}</span>
                </div>
            </footer>
        </div>
    }
}

// Helper function to render challenges list
fn render_challenges_list(
    challenge_mode: UseStateHandle<bool>,
    current_challenge_id: UseStateHandle<Option<u32>>,
    program_code: UseStateHandle<String>,
    challenges_open: UseStateHandle<bool>,
) -> Html {
    html! {
        <div class="challenges-list">
            <h3>{"Available Challenges"}</h3>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(1));
                            program_code.set(CHALLENGE_1_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 1"}
                </button>
                <p><strong>{"Challenge 1: Load and Halt"}</strong></p>
                <p>{"Load the number 42 into register x1, then halt."}</p>
            </div>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(2));
                            program_code.set(CHALLENGE_2_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 2"}
                </button>
                <p><strong>{"Challenge 2: Simple Addition"}</strong></p>
                <p>{"Add 5 + 3 and store the result in register x1."}</p>
            </div>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(3));
                            program_code.set(CHALLENGE_3_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 3"}
                </button>
                <p><strong>{"Challenge 3: Register Transfer"}</strong></p>
                <p>{"Copy the value from register x1 to register x2 (x1 starts with 100)."}</p>
            </div>
        </div>
    }
}

// Constants for content
const EXAMPLE_PROGRAM: &str = "; Example: Add two numbers
; This program demonstrates basic RISC-V assembly

; Load 5 into x1
ADDI x1, x0, 5

; Load 10 into x2
ADDI x2, x0, 10

; Add x1 + x2 -> x3
ADD x3, x1, x2

; Halt the CPU
HALT";

const CHALLENGE_1_TEMPLATE: &str = "; Challenge 1: Load and Halt
; Load the number 42 into register x1, then halt

; Your code here
";

const CHALLENGE_2_TEMPLATE: &str = "; Challenge 2: Simple Addition
; Add 5 + 3 and store the result in register x1

; Your code here
";

const CHALLENGE_3_TEMPLATE: &str = "; Challenge 3: Register Transfer
; Copy the value from register x1 to register x2
; x1 starts with the value 100

; Your code here
";

const TUTORIAL_CONTENT: &str = r#"
<h3>Welcome to the RISC-V Assembly Game!</h3>
<p>This game teaches you assembly programming using a simplified RISC-V CPU.</p>

<h4>CPU Features:</h4>
<ul>
    <li><strong>8 Registers</strong>: x0 through x7 (x0 is always 0)</li>
    <li><strong>1KB Memory</strong>: For storing instructions and data</li>
    <li><strong>Rich Instruction Set</strong>: Arithmetic, logic, memory, branches, and jumps</li>
</ul>

<h4>Basic Instructions:</h4>
<ul>
    <li><code>ADDI rd, rs1, imm</code> - Add immediate value to register</li>
    <li><code>ADD rd, rs1, rs2</code> - Add two registers</li>
    <li><code>SUB rd, rs1, rs2</code> - Subtract registers</li>
    <li><code>AND/OR/XOR rd, rs1, rs2</code> - Bitwise operations</li>
    <li><code>LW/LB rd, offset(rs1)</code> - Load from memory</li>
    <li><code>SW/SB rs2, offset(rs1)</code> - Store to memory</li>
    <li><code>BEQ/BNE/BLT/BGE rs1, rs2, offset</code> - Branch instructions</li>
    <li><code>HALT</code> - Stop execution</li>
</ul>

<h4>Example:</h4>
<pre>// Load 5 into x1
ADDI x1, x0, 5

// Load 10 into x2
ADDI x2, x0, 10

// Add x1 + x2 -> x3
ADD x3, x1, x2

// Halt
HALT</pre>
"#;

const ISA_REF_CONTENT: &str = r#"
<h3>RV32 Instruction Set Reference</h3>

<h4>Arithmetic Instructions</h4>

<p><strong>ADDI rd, rs1, imm</strong> - Add Immediate</p>
<p>Operation: <code>rd = rs1 + imm</code></p>
<p>Example: <code>ADDI x1, x0, 42</code> loads 42 into x1</p>

<p><strong>ADD rd, rs1, rs2</strong> - Add Registers</p>
<p>Operation: <code>rd = rs1 + rs2</code></p>
<p>Example: <code>ADD x3, x1, x2</code> adds x1 and x2, stores in x3</p>

<p><strong>SUB rd, rs1, rs2</strong> - Subtract Registers</p>
<p>Operation: <code>rd = rs1 - rs2</code></p>
<p>Example: <code>SUB x3, x1, x2</code> computes x1 - x2, stores in x3</p>

<h4>Logical Instructions</h4>

<p><strong>AND rd, rs1, rs2</strong> - Bitwise AND</p>
<p><strong>ANDI rd, rs1, imm</strong> - Bitwise AND with immediate</p>
<p><strong>OR rd, rs1, rs2</strong> - Bitwise OR</p>
<p><strong>ORI rd, rs1, imm</strong> - Bitwise OR with immediate</p>
<p><strong>XOR rd, rs1, rs2</strong> - Bitwise XOR</p>
<p><strong>XORI rd, rs1, imm</strong> - Bitwise XOR with immediate</p>

<h4>Memory Instructions</h4>

<p><strong>LW rd, offset(rs1)</strong> - Load Word (4 bytes)</p>
<p>Example: <code>LW x1, 4(x2)</code></p>

<p><strong>LB rd, offset(rs1)</strong> - Load Byte (1 byte, sign-extended)</p>
<p>Example: <code>LB x1, 0(x2)</code></p>

<p><strong>SW rs2, offset(rs1)</strong> - Store Word (4 bytes)</p>
<p>Example: <code>SW x1, 8(x2)</code></p>

<p><strong>SB rs2, offset(rs1)</strong> - Store Byte (1 byte)</p>
<p>Example: <code>SB x1, 0(x2)</code></p>

<h4>Branch Instructions</h4>

<p><strong>BEQ rs1, rs2, offset</strong> - Branch if Equal</p>
<p><strong>BNE rs1, rs2, offset</strong> - Branch if Not Equal</p>
<p><strong>BLT rs1, rs2, offset</strong> - Branch if Less Than</p>
<p><strong>BGE rs1, rs2, offset</strong> - Branch if Greater or Equal</p>

<h4>Jump Instructions</h4>

<p><strong>J offset</strong> - Unconditional Jump</p>
<p><strong>JAL rd, offset</strong> - Jump and Link (save return address)</p>
<p><strong>JALR rd, offset(rs1)</strong> - Jump and Link Register</p>

<h4>Special</h4>

<p><strong>HALT</strong> - Stop Execution</p>
<p>Halts program execution. Always end your programs with HALT!</p>

<p><strong>NOP</strong> - No Operation</p>
<p>Does nothing, advances PC.</p>

<h4>Register x0</h4>
<p><strong>Special Property:</strong> Always contains the value 0</p>
<p><strong>Use:</strong> Can be used as a source to get zero, or as a destination to discard results</p>
"#;

const HELP_CONTENT: &str = r#"
<h3>Help & Tips</h3>

<h4>How to Use:</h4>
<ol>
    <li><strong>Write Code</strong>: Enter your program in the editor using assembly mnemonics (ADDI, ADD, etc.)</li>
    <li><strong>Assemble</strong>: Click "Assemble" to parse and load your program into the CPU</li>
    <li><strong>Step/Run</strong>: Use "Step" to execute one instruction or "Run" to complete the program</li>
    <li><strong>Reset</strong>: Click "Reset" to clear the CPU and start over</li>
</ol>

<h4>Challenges:</h4>
<p>Click "Challenges" to see available programming puzzles. Each challenge has specific requirements and test cases.</p>

<h4>Debugging Tips:</h4>
<ul>
    <li>Use "Step" to execute your program one instruction at a time</li>
    <li>Watch the registers panel to see how values change</li>
    <li>Check the memory viewer to see your program in memory</li>
    <li>The assembly output shows success/error messages</li>
</ul>

<h4>Common Mistakes:</h4>
<ul>
    <li>Forgetting to include HALT (99) at the end</li>
    <li>Using wrong opcode numbers</li>
    <li>Wrong number of operands for an instruction</li>
    <li>Using register numbers outside 0-7</li>
</ul>
"#;
