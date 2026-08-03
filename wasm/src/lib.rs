use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use akron::error::RuntimeError;
use akron::specs::CapabilitySet;
use akron::{HostPolicy, ValueOps, VM};
use akronc::{CompileOptions, Compiler};
use memory::Value;

// Thread-local buffer for capturing print() output.
thread_local! {
    static OUTPUT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Custom print native that writes to the thread-local buffer instead of stdout.
fn captured_print(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
    let mut line = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        line.push_str(&vm.val_to_string(arg));
    }
    OUTPUT.with(|buf| buf.borrow_mut().push(line));
    Ok(Value::nil())
}

/// Result of running an Achronyme program.
#[wasm_bindgen]
pub struct RunResult {
    success: bool,
    output: String,
    error: String,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> String {
        self.error.clone()
    }
}

/// Compile and run an Achronyme source program.
///
/// Returns a `RunResult` with captured output, success status, and error message.
#[wasm_bindgen]
pub fn run(source: &str) -> RunResult {
    // Clear the output buffer
    OUTPUT.with(|buf| buf.borrow_mut().clear());

    match run_inner(source) {
        Ok(()) => {
            let output = OUTPUT.with(|buf| buf.borrow().join("\n"));
            RunResult {
                success: true,
                output,
                error: String::new(),
            }
        }
        Err(msg) => {
            let output = OUTPUT.with(|buf| buf.borrow().join("\n"));
            RunResult {
                success: false,
                output,
                error: msg,
            }
        }
    }
}

fn run_inner(source: &str) -> Result<(), String> {
    let native_table = achronyme_std::std_native_table();
    let mut compiler = Compiler::with_extra_natives(&native_table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .map_err(|error| error.to_string())?;

    let mut vm = VM::new();
    vm.host_policy = HostPolicy::untrusted();
    // Browser output is an explicit virtual capability. Filesystem, network,
    // clock, randomness, and console input remain unavailable.
    vm.host_policy.grant(CapabilitySet::CONSOLE_WRITE);
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module)
            .map_err(|error| error.to_string())?;
    }
    vm.host_policy
        .require_program(program.requested_host_capabilities())
        .map_err(|error| format!("Host capability preflight failed: {error}"))?;

    if !vm.natives.is_empty() {
        let mut print = vm.natives[0].clone();
        print.func = captured_print;
        vm.natives[0] = print;
    }
    vm.load_program(program)
        .map_err(|error| error.to_string())?;
    vm.interpret().map_err(|e| {
        if let Some((func_name, line)) = &vm.last_error_location {
            format!("[line {line}] in {func_name}: {e}")
        } else {
            format!("Runtime error: {e}")
        }
    })
}

/// Explicit support contract for the browser runtime.
///
/// Pure structured tasks, bounded channels, and cooperative yield run inside
/// the single-lane VM. Host I/O remains unavailable until an embedding API
/// supplies both a target adapter and explicit authority.
#[wasm_bindgen]
pub fn runtime_support() -> String {
    serde_json::json!({
        "ambient_authority": false,
        "virtual_console_output": {
            "status": "supported",
            "adapter": "captured browser output"
        },
        "structured_tasks": {
            "status": "supported",
            "adapter": "single-lane cooperative VM scheduler"
        },
        "channels": {
            "status": "supported",
            "adapter": "bounded in-memory VM channels"
        },
        "yield_now": {
            "status": "supported",
            "adapter": "cooperative VM ready queue"
        },
        "timers": {
            "status": "unsupported",
            "reason": "no explicit WASM clock adapter"
        },
        "files": {
            "status": "unsupported",
            "reason": "no explicit WASM file adapter"
        },
        "network": {
            "status": "unsupported",
            "reason": "no explicit WASM network adapter"
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// LSP functions (powered by ach-lsp-core)
// ---------------------------------------------------------------------------

/// Check `.ach` source code for diagnostics. Returns JSON array of LspDiagnostic[].
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    let diags = ach_lsp_core::diagnostics::check(source);
    serde_json::to_string(&diags).unwrap_or_else(|_| "[]".into())
}

/// Check `.circom` source code for diagnostics. Returns JSON array of LspDiagnostic[].
///
/// Routes through the circom parser + constraint analyzer + (for self-contained
/// sources) the lowering pipeline. Use this entry point — not [`check`] — for
/// any URI ending in `.circom`. The two pipelines surface different code
/// families (E100-E102 / W101-W103 / E200-E211 vs the `.ach` parser codes).
#[wasm_bindgen]
pub fn check_circom(source: &str) -> String {
    let diags = ach_lsp_core::diagnostics_circom::check_circom(source);
    serde_json::to_string(&diags).unwrap_or_else(|_| "[]".into())
}

/// Get all `.ach` completion items. Returns JSON array of CompletionItem[].
/// This is static data (keywords + builtins + snippets), no source needed.
#[wasm_bindgen]
pub fn completions() -> String {
    let mut items = ach_lsp_core::completion::keyword_completions();
    items.extend(ach_lsp_core::completion::snippet_completions());
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
}

/// Canonical language metadata for editors and browser tooling.
#[wasm_bindgen]
pub fn language_metadata() -> String {
    let keywords = achronyme_parser::token::KEYWORDS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    let effects = akron::specs::EFFECT_CATALOG
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    let capabilities = akron::specs::CAPABILITY_CATALOG
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    let builtins = resolve::BuiltinRegistry::default()
        .entries()
        .iter()
        .map(|entry| {
            serde_json::json!({
                "name": entry.name,
                "arity": entry.arity.describe(),
                "availability": entry.availability.to_string(),
                "effects": entry.effects.to_string(),
                "capabilities": entry.capabilities.to_string(),
                "behavior": match entry.behavior {
                    resolve::NativeBehavior::Immediate => "immediate",
                    resolve::NativeBehavior::Blocking => "blocking",
                    resolve::NativeBehavior::Suspending => "suspending",
                },
                "cancellation": match entry.cancellation {
                    resolve::CancellationPolicy::None => "none",
                    resolve::CancellationPolicy::BeforeStart => "before-start",
                    resolve::CancellationPolicy::Cooperative => "cooperative",
                },
                "resource": entry.resource.to_bytes(),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "keywords": keywords,
        "effects": effects,
        "capabilities": capabilities,
        "builtins": builtins,
    })
    .to_string()
}

/// Get all `.circom` completion items. Returns JSON array of CompletionItem[].
///
/// Disjoint from [`completions`] — circom keywords (`template`, `signal`,
/// `pragma`, `include`) and the verified circomlib templates (Num2Bits,
/// Poseidon, MiMCSponge, …) live in the circom-specific tables.
#[wasm_bindgen]
pub fn completions_circom() -> String {
    let mut items = ach_lsp_core::completion::circom_keyword_completions();
    items.extend(ach_lsp_core::completion::circom_snippet_completions());
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
}

/// Get `.ach` hover documentation for the word at (line, col).
/// Returns a markdown string or `""` when no entry matches.
#[wasm_bindgen]
pub fn hover(source: &str, line: u32, col: u32) -> String {
    let word = match ach_lsp_core::document::word_at_position(source, line, col) {
        Some((w, _)) => w,
        None => return String::new(),
    };
    ach_lsp_core::hover::hover_for(&word)
        .unwrap_or("")
        .to_string()
}

/// Get `.circom` hover documentation for the word at (line, col).
/// Returns a markdown string or `""` when no entry matches.
///
/// `Poseidon`, `Pedersen`, `Sha256`, etc. resolve to circomlib component
/// docs here; in the `.ach` table the same identifiers (when present)
/// resolve to the achronyme builtin instead.
#[wasm_bindgen]
pub fn hover_circom(source: &str, line: u32, col: u32) -> String {
    let word = match ach_lsp_core::document::word_at_position(source, line, col) {
        Some((w, _)) => w,
        None => return String::new(),
    };
    ach_lsp_core::hover::circom_hover_for(&word)
        .unwrap_or("")
        .to_string()
}

/// Go to definition for the word at (line, col). Returns JSON Range or "".
#[wasm_bindgen]
pub fn goto_definition(source: &str, line: u32, col: u32) -> String {
    let byte_offset = match ach_lsp_core::definitions::position_to_byte_offset(source, line, col) {
        Some(o) => o,
        None => return String::new(),
    };
    match ach_lsp_core::definitions::goto_definition(source, byte_offset) {
        Some(range) => serde_json::to_string(&range).unwrap_or_default(),
        None => String::new(),
    }
}

/// Extract document symbols. Returns JSON array of DocumentSymbol[].
#[wasm_bindgen]
pub fn document_symbols(source: &str) -> String {
    let syms = ach_lsp_core::symbols::document_symbols(source);
    serde_json::to_string(&syms).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_metadata_includes_concurrency_and_effect_catalogs() {
        let metadata: serde_json::Value = serde_json::from_str(&language_metadata()).unwrap();
        assert!(metadata["keywords"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("concurrent")));
        assert!(metadata["effects"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("task")));
        assert!(metadata["capabilities"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("network.listen")));
    }

    #[test]
    fn wasm_runtime_support_matrix_is_explicit() {
        let support: serde_json::Value = serde_json::from_str(&runtime_support()).unwrap();
        assert_eq!(support["ambient_authority"], false);
        assert_eq!(support["structured_tasks"]["status"], "supported");
        assert_eq!(support["channels"]["status"], "supported");
        assert_eq!(support["timers"]["status"], "unsupported");
        assert_eq!(support["files"]["status"], "unsupported");
        assert_eq!(support["network"]["status"], "unsupported");
    }

    #[test]
    fn wasm_runner_supports_pure_tasks_channels_and_yield() {
        let result = run(
            r#"
                fn producer(messages) {
                    await yield_now()
                    await channel_send(messages, "wasm")
                }
                let messages = channel(1)
                let value = concurrent {
                    spawn producer(messages)
                    await channel_receive(messages)
                }
                print(value)
            "#,
        );
        assert!(result.success(), "{}", result.error());
        assert_eq!(result.output(), "wasm");
    }

    #[test]
    fn wasm_runner_rejects_unadapted_host_io_before_execution() {
        for (source, capability) in [
            ("await sleep(1)", "clock"),
            ("await open_file(\"data.txt\")", "file.read"),
            ("await tcp_connect(\"127.0.0.1:9\")", "network.connect"),
        ] {
            let result = run(source);
            assert!(!result.success(), "{source}");
            assert!(
                result.error().contains("Host capability preflight failed"),
                "{}",
                result.error()
            );
            assert!(result.error().contains(capability), "{}", result.error());
        }
    }

    #[test]
    fn wasm_runner_preflights_ambient_clock_authority() {
        let result = run("return time()");
        assert!(!result.success());
        assert!(result.error().contains("clock"), "{}", result.error());
    }
}
