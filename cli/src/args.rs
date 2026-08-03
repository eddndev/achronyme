use clap::{Parser, Subcommand};
use cli::commands::engine::ExecutionEngine;

#[derive(Parser)]
#[command(name = "ach", version)]
#[command(about = "Achronyme CLI", long_about = None)]
pub struct Cli {
    /// Diagnostic output format: human (default), json, or short
    #[arg(long, global = true)]
    pub error_format: Option<String>,

    /// Prime field: bn254 (default), bls12-381, or goldilocks
    #[arg(long, global = true)]
    pub prime: Option<String>,

    /// Do not load achronyme.toml project configuration
    #[arg(long, global = true)]
    pub no_config: bool,

    /// Permit development-only single-party proving-key setup
    #[arg(long, global = true, conflicts_with = "trusted_key_dir")]
    pub insecure_dev_setup: bool,

    /// Directory containing ceremony-derived proving-key artifacts
    #[arg(
        long,
        value_name = "DIR",
        global = true,
        conflicts_with = "insecure_dev_setup"
    )]
    pub trusted_key_dir: Option<String>,

    /// Grant read-only access to a directory (repeatable)
    #[arg(long = "allow-read", value_name = "DIR", global = true)]
    pub allow_read: Vec<String>,

    /// Grant write access to a directory (repeatable)
    #[arg(long = "allow-write", value_name = "DIR", global = true)]
    pub allow_write: Vec<String>,

    /// Grant outbound TCP access to one numeric IP:port (repeatable)
    #[arg(long = "allow-connect", value_name = "IP:PORT", global = true)]
    pub allow_connect: Vec<String>,

    /// Grant TCP listen access to one numeric IP:port (repeatable)
    #[arg(long = "allow-listen", value_name = "IP:PORT", global = true)]
    pub allow_listen: Vec<String>,

    /// Maximum number of live child tasks
    #[arg(long, global = true)]
    pub max_tasks: Option<usize>,

    /// Maximum number of simultaneously open owned resources
    #[arg(long, global = true)]
    pub max_resources: Option<usize>,

    /// Maximum nesting depth of concurrent task scopes
    #[arg(long, global = true)]
    pub max_task_scopes: Option<usize>,

    /// Maximum number of in-flight asynchronous native requests
    #[arg(long, global = true)]
    pub max_pending_native_requests: Option<usize>,

    /// Maximum number of completed task results retained for awaiting handles
    #[arg(long, global = true)]
    pub max_retained_task_results: Option<usize>,

    /// Maximum pending bounded-channel operations
    #[arg(long, global = true)]
    pub max_channel_operations: Option<usize>,

    /// Maximum number of simultaneously open bounded channels
    #[arg(long, global = true)]
    pub max_channels: Option<usize>,

    /// Number of workers in the bounded blocking I/O pool
    #[arg(long, global = true)]
    pub blocking_workers: Option<usize>,

    /// Capacity of the bounded blocking I/O request queue
    #[arg(long, global = true)]
    pub blocking_queue_capacity: Option<usize>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Achronyme project
    Init {
        /// Project name
        name: String,
        /// Template: "circuit" (default), "vm", or "prove"
        #[arg(long, default_value = "circuit")]
        template: String,
    },
    /// Run a source file or binary
    Run {
        /// Path to the file (.ach or .achb). If omitted, uses [project].entry from achronyme.toml
        path: Option<String>,
        /// Force GC on every allocation (Stress Mode)
        #[arg(long)]
        stress_gc: bool,
        /// [Deprecated] Ignored — native Groth16 backend does not use ptau files
        #[arg(long, hide = true)]
        ptau: Option<String>,
        /// Backend for prove {} blocks: "r1cs" (default) or "plonkish"
        #[arg(long)]
        prove_backend: Option<String>,
        /// Maximum heap size (e.g., "256M", "1G", "512K", or raw bytes)
        #[arg(long)]
        max_heap: Option<String>,
        /// Maximum number of bytecode instructions to execute
        #[arg(long)]
        max_instructions: Option<u64>,
        /// Print GC statistics to stderr after execution
        #[arg(long)]
        gc_stats: bool,
        /// Print circuit constraint stats for each prove block
        #[arg(long)]
        circuit_stats: bool,
        /// Execution engine: interpreter, jit, or auto
        #[arg(long, value_enum, default_value_t = ExecutionEngine::default())]
        engine: ExecutionEngine,
    },
    /// Disassemble a source file or binary
    Disassemble {
        /// Path to the file. If omitted, uses [project].entry from achronyme.toml
        path: Option<String>,
    },
    /// Compile a source file to binary
    Compile {
        /// Input source file. If omitted, uses [project].entry from achronyme.toml
        path: Option<String>,
        /// Output binary file (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Compile a source file or current ACHB image to a native executable
    Aot {
        /// Input source or ACHB file. If omitted, uses [project].entry
        path: Option<String>,
        /// Native executable path
        #[arg(short, long)]
        output: Option<String>,
        /// Path to libakron_aot_runtime.a
        #[arg(long)]
        runtime: Option<String>,
        /// Clang 21 executable
        #[arg(long, default_value = "clang")]
        clang: String,
    },
    /// Open the interactive circuit inspector in the browser
    Inspect {
        /// Path to the source file (.ach). If omitted, uses [project].entry from achronyme.toml
        path: Option<String>,
        /// Name of the prove block to inspect (runs program via VM to resolve captures)
        #[arg(long)]
        prove: Option<String>,
        /// Input values as name=value pairs (comma-separated, for standalone circuits)
        #[arg(long)]
        inputs: Option<String>,
        /// Input values from a TOML file (for standalone circuits)
        #[arg(long)]
        input_file: Option<String>,
        /// HTTP server port (default: 3000)
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Address to bind the HTTP server to. Defaults to 127.0.0.1 (loopback only).
        /// The inspector exposes witness values, source, and DAG state without auth —
        /// binding to 0.0.0.0 or a LAN IP makes that visible to every peer on the network.
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// Don't auto-open the browser
        #[arg(long)]
        no_open: bool,
        /// Print the program effect/capability manifest and exit
        #[arg(long)]
        manifest: bool,
    },
    /// Compile a .circom file and optionally generate a proof
    Circom {
        /// Path to the .circom file
        path: Option<String>,
        /// Input values as name=value pairs (comma-separated, decimal or 0x hex)
        #[arg(long)]
        inputs: Option<String>,
        /// Input values from a TOML file. May be repeated with the r1cs
        /// backend: the circuit is compiled once and each extra file
        /// produces its own witness/proof, outputs suffixed by file stem
        #[arg(long)]
        input_file: Vec<String>,
        /// Disable IR optimization passes
        #[arg(long)]
        no_optimize: Option<bool>,
        /// Backend: "r1cs" (default) or "plonkish"
        #[arg(long)]
        backend: Option<String>,
        /// Generate a cryptographic proof (requires --inputs)
        #[arg(long)]
        prove: bool,
        /// Output .r1cs file path
        #[arg(long)]
        r1cs: Option<String>,
        /// Output .wtns file path
        #[arg(long)]
        wtns: Option<String>,
        /// Generate a Solidity Groth16 verifier contract at the given path
        #[arg(long)]
        solidity: Option<String>,
        /// Export Plonkish circuit to JSON (includes witness if --inputs is provided)
        #[arg(long)]
        plonkish_json: Option<String>,
        /// Dump the SSA IR (after optimization) and exit without compiling to constraints
        #[arg(long)]
        dump_ir: bool,
        /// Print circuit constraint stats breakdown
        #[arg(long)]
        circuit_stats: bool,
        /// Library directories for include resolution (like Circom -l flag)
        #[arg(short = 'l', long = "lib")]
        lib_dirs: Vec<String>,
    },
    /// Compile a circuit to .r1cs (and optionally generate .wtns)
    Circuit {
        /// Path to the source file (.ach). If omitted, uses [project].entry from achronyme.toml
        path: Option<String>,
        /// Output .r1cs file path
        #[arg(long)]
        r1cs: Option<String>,
        /// Output .wtns file path
        #[arg(long)]
        wtns: Option<String>,
        /// Input values as name=value pairs (comma-separated, decimal or 0x hex)
        #[arg(long)]
        inputs: Option<String>,
        /// Input values from a TOML file (arrays supported natively)
        #[arg(long)]
        input_file: Option<String>,
        /// Disable IR optimization passes
        #[arg(long)]
        no_optimize: Option<bool>,
        /// Backend: "r1cs" (default) or "plonkish"
        #[arg(long)]
        backend: Option<String>,
        /// Generate a cryptographic proof (requires --inputs)
        #[arg(long)]
        prove: bool,
        /// Generate a Solidity Groth16 verifier contract at the given path
        #[arg(long)]
        solidity: Option<String>,
        /// Export Plonkish circuit to JSON (includes witness if --inputs is provided)
        #[arg(long)]
        plonkish_json: Option<String>,
        /// Dump the SSA IR (after optimization) and exit without compiling to constraints
        #[arg(long)]
        dump_ir: bool,
        /// Print circuit constraint stats breakdown
        #[arg(long)]
        circuit_stats: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proving_key_flags_are_mutually_exclusive() {
        let parsed = Cli::try_parse_from([
            "ach",
            "--insecure-dev-setup",
            "--trusted-key-dir",
            "keys",
            "run",
            "program.ach",
        ]);

        assert!(parsed.is_err());
    }
}
