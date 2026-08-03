use crate::error::RuntimeError;
use crate::globals::GlobalEntry;
use crate::host::HostPolicy;
use crate::native::NativeObj;
use crate::resource::ResourceTable;
use memory::field::PrimeId;
use memory::{Heap, Value};
use std::collections::HashMap;

use super::blocking_pool::BlockingPool;
use super::channel_hub::ChannelHub;
use super::circom::CircomWitnessHandler;
use super::frame::CallFrame;
use super::interpreter::InterpreterOps;
use super::limits::RuntimeLimits;
use super::native::NativeRegistry;
use super::network_reactor::NetworkReactor;
use super::prototype::PrototypeRegistry;
use super::prove::{ProveHandler, VerifyHandler};
use super::task::TaskScheduler;
use super::timer_reactor::TimerReactor;
use super::upvalue::UpvalueOps;

/// The Virtual Machine struct
pub struct VM {
    pub heap: Heap,
    // SAFETY: Box<[Value]> guarantees stable address (no reallocation)
    pub stack: Box<[Value]>,
    pub frames: Vec<CallFrame>,
    pub globals: Vec<GlobalEntry>,
    pub interner: HashMap<String, u32>,
    pub natives: Vec<NativeObj>,

    /// Function prototypes (handles) for O(1) CLOSURE lookup
    pub prototypes: Vec<u32>,

    /// Head of the linked list of open upvalues (Index into Heap.upvalues)
    pub open_upvalues: Option<u32>,

    /// If true, GC will run on every possible occasion (for testing)
    pub stress_mode: bool,

    /// Instruction budget. Each executed instruction decrements the counter.
    /// Execution stops with `RuntimeError::InstructionBudgetExhausted` when
    /// it reaches zero. `u64::MAX` means unlimited (no budget).
    pub instruction_budget: u64,

    /// Explicit, inspectable bounds for task and resource growth.
    pub runtime_limits: RuntimeLimits,

    // Passive Debug Symbols (Sidecar)
    pub debug_symbols: Option<HashMap<u16, String>>,

    /// Handler for `prove { }` blocks (injected by CLI or host)
    pub prove_handler: Option<Box<dyn ProveHandler>>,

    /// Handler for `verify_proof()` calls (injected by CLI or host)
    pub verify_handler: Option<Box<dyn VerifyHandler>>,

    /// Handler for `CallCircomTemplate` opcode — evaluates a circom
    /// template at runtime via the circom frontend. Injected by the
    /// CLI after compile time so the VM can dispatch into an
    /// in-process library registry without `vm` depending on `circom`.
    pub circom_handler: Option<Box<dyn CircomWitnessHandler>>,

    /// Location of the last runtime error: (function_name, line_number).
    /// Set by interpret() before returning Err.
    pub last_error_location: Option<(String, u32)>,

    /// Value returned by the most recently completed top-level frame.
    pub last_result: Value,

    /// GC roots for values held by native functions during reentrant calls.
    ///
    /// Higher-order natives (map, filter, reduce, etc.) re-enter the
    /// interpreter via `call_value()`. Intermediate results live in Rust
    /// locals that the GC cannot see. Pushing them here keeps them rooted
    /// across closure invocations that may trigger garbage collection.
    pub native_roots: Vec<Value>,

    /// Per-tag method tables for method dispatch (`.method()` syntax).
    pub prototype_registry: PrototypeRegistry,

    /// Prime field loaded from bytecode header (v0x0B+). Defaults to BN254.
    pub prime_id: PrimeId,

    /// Deterministic, single-lane structured task scheduler.
    pub(crate) task_scheduler: TaskScheduler,

    /// Explicit authority granted by the embedding host.
    pub host_policy: HostPolicy,

    /// Fixed-size worker pool used only by explicitly registered blocking
    /// native adapters.
    pub(crate) blocking_pool: BlockingPool,

    /// Bounded, single-lane channel queues and waiters.
    pub(crate) channel_hub: ChannelHub,

    /// VM-owned opaque host resources. Handles are monotonic and never reused.
    pub(crate) resources: ResourceTable,

    /// One readiness reactor per VM; network tasks never require one OS thread
    /// apiece.
    pub(crate) network_reactor: NetworkReactor,

    /// One bounded timer service per VM, shared by every sleep and deadline.
    pub(crate) timer_reactor: TimerReactor,

    /// Prevents suspension from crossing a non-resumable Rust native frame.
    pub(crate) native_call_depth: usize,
}

pub const STACK_MAX: usize = 65_536;
pub const MAX_FRAMES: usize = 4096;

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new VM instance with bootstrapped native functions
    pub fn new() -> Self {
        // Pre-allocate fixed-size stack and PIN it via Box
        let stack = vec![Value::nil(); STACK_MAX].into_boxed_slice();

        let mut vm = Self {
            heap: Heap::new(),
            stack,
            frames: Vec::with_capacity(64),
            globals: Vec::with_capacity(64),
            interner: HashMap::new(),
            natives: Vec::new(),
            prototypes: Vec::new(),
            open_upvalues: None,
            stress_mode: false,
            instruction_budget: u64::MAX,
            runtime_limits: RuntimeLimits::default(),
            debug_symbols: None,
            prove_handler: None,
            verify_handler: None,
            circom_handler: None,
            last_error_location: None,
            last_result: Value::nil(),
            native_roots: Vec::new(),
            prototype_registry: PrototypeRegistry::new(),
            prime_id: PrimeId::Bn254,
            task_scheduler: TaskScheduler::default(),
            host_policy: HostPolicy::default(),
            blocking_pool: BlockingPool::new(),
            channel_hub: ChannelHub::default(),
            resources: ResourceTable::default(),
            network_reactor: NetworkReactor::new(),
            timer_reactor: TimerReactor::new(),
            native_call_depth: 0,
        };

        // Bootstrap native functions and prototype methods
        vm.bootstrap_natives()
            .expect("bootstrap_natives: arena allocation failed at startup");
        vm.prototype_registry.bootstrap();

        vm
    }

    pub(crate) fn finish_compiled_call(
        &mut self,
        frame_index: usize,
        result: Value,
    ) -> Result<(), RuntimeError> {
        if frame_index + 1 != self.frames.len() {
            return Err(RuntimeError::StackUnderflow);
        }
        let frame = self
            .frames
            .get(frame_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;
        self.close_upvalues(frame.base)?;
        self.frames.pop();
        if self.frames.is_empty() {
            self.last_result = result;
        } else {
            let destination = self
                .stack
                .get_mut(frame.dest_reg)
                .ok_or(RuntimeError::StackOverflow)?;
            *destination = result;
        }
        Ok(())
    }

    /// Finish a compiled main frame through the canonical VM return path.
    pub fn finish_compiled_top_level(&mut self, result: Value) -> Result<(), RuntimeError> {
        self.finish_compiled_call(0, result)
    }

    /// Register an external `NativeModule` after bootstrap.
    ///
    /// Used by the CLI to add stdlib modules (`achronyme-std`) whose
    /// indices continue after the builtins. The compiler must have been
    /// initialized with the same extra natives via `with_extra_natives()`
    /// so that global indices align.
    pub fn register_module(
        &mut self,
        module: &dyn crate::module::NativeModule,
    ) -> Result<(), RuntimeError> {
        use crate::machine::native::NativeRegistry;
        for def in module.natives() {
            self.define_native(&def)?;
        }
        Ok(())
    }

    /// Invoke a callable Value (Closure or Native) with the given arguments.
    ///
    /// Used by higher-order native functions (map, filter, reduce, etc.) to
    /// re-enter the interpreter loop and execute user-provided closures.
    ///
    /// For closures, this pushes a CallFrame, runs the interpreter until the
    /// closure returns, and yields the return value. GC runs normally during
    /// execution — callers must root any intermediate heap values via
    /// `self.native_roots` before invoking this method.
    ///
    /// For natives, this delegates directly to the function pointer.
    pub fn call_value(&mut self, callee: Value, args: &[Value]) -> Result<Value, RuntimeError> {
        // Fast path: native-to-native call (no frame push needed)
        if callee.is_native() {
            let handle = callee
                .as_handle()
                .ok_or_else(|| RuntimeError::type_mismatch("Expected native handle"))?;
            return self.invoke_native(handle, args);
        }

        if !callee.is_closure() {
            return Err(RuntimeError::type_mismatch(
                "call_value target must be a Closure or Native",
            ));
        }

        let closure_handle = callee
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("Expected closure handle"))?;

        let (arity, max_slots) = {
            let closure = self
                .heap
                .get_closure(closure_handle)
                .ok_or(RuntimeError::FunctionNotFound)?;
            let func = self
                .heap
                .get_function(closure.function)
                .ok_or(RuntimeError::FunctionNotFound)?;
            (func.arity as usize, func.max_slots as usize)
        };

        if arity != args.len() {
            return Err(RuntimeError::arity_mismatch(format!(
                "Expected {} args, got {}",
                arity,
                args.len()
            )));
        }

        // Compute the next safe stack base above all active frames.
        let mut new_base = 0usize;
        for frame in &self.frames {
            if let Some(cl) = self.heap.get_closure(frame.closure) {
                if let Some(f) = self.heap.get_function(cl.function) {
                    new_base = new_base.max(frame.base + f.max_slots as usize);
                }
            }
        }

        if new_base + max_slots >= STACK_MAX {
            return Err(RuntimeError::StackOverflow);
        }

        // Copy arguments into the new frame's register window.
        for (i, arg) in args.iter().enumerate() {
            self.stack[new_base + i] = *arg;
        }

        // dest_reg: the Return opcode writes the closure's return value here.
        // We reuse new_base (R0 of the new frame) so we can read it back after
        // the frame is popped.
        let dest_reg = new_base;

        if self.frames.len() >= MAX_FRAMES {
            return Err(RuntimeError::StackOverflow);
        }

        let saved_depth = self.frames.len();
        self.frames
            .push(CallFrame::new(closure_handle, new_base, dest_reg));

        match self.run_until_frame_depth(saved_depth) {
            Ok(()) if saved_depth == 0 => Ok(self.last_result),
            Ok(()) => Ok(self.stack[dest_reg]),
            Err(RuntimeError::TaskSuspended) => Err(RuntimeError::TaskSuspended),
            Err(e) => {
                self.capture_error_location();
                // Clean up any frames left by a failed closure (e.g. error mid-execution
                // of a closure that called other closures).
                self.frames.truncate(saved_depth);
                Err(e)
            }
        }
    }

    /// Import compiler strings into the VM heap.
    ///
    /// This replaces the heap's entire string arena with the compiler's
    /// string table and clears the VM's string interner (whose handle
    /// mappings become stale after the arena swap).
    ///
    /// # Panics
    ///
    /// Panics if called after execution has started (the string arena's
    /// free list is non-empty, indicating GC has already run).
    pub fn import_strings(&mut self, strings: Vec<String>) {
        self.heap.import_strings(strings);
        self.interner.clear();
    }

    /// Soft Reset: Clears stack and frames for running a new script.
    /// CRITICAL: Must close all open upvalues to prevent them from pointing to dead stack slots.
    pub fn reset(&mut self) {
        // 1. Close ALL open upvalues (stack index 0 = everything)
        // Ignore errors here — reset drains the upvalue list regardless.
        let _ = self.close_upvalues(0);

        // 2. Clear Runtime State
        self.frames.clear();
        self.open_upvalues = None;
        self.last_result = Value::nil();
        self.last_error_location = None;
        self.task_scheduler.reset();
        self.native_call_depth = 0;
        self.close_all_resources();
        self.channel_hub = ChannelHub::default();

        // 3. Zero stack in debug builds to prevent stale values leaking
        #[cfg(debug_assertions)]
        self.stack.fill(Value::nil());
    }

    /// Capture the current execution location for error reporting.
    pub(crate) fn capture_error_location(&mut self) {
        if let Some(location) = self.current_source_location().filter(|(_, line)| *line > 0) {
            self.last_error_location = Some(location);
        }
    }

    pub(crate) fn current_source_location(&self) -> Option<(String, u32)> {
        let frame = self.frames.last()?;
        let ip = frame.ip.saturating_sub(1);
        let closure = self.heap.get_closure(frame.closure)?;
        let function = self.heap.get_function(closure.function)?;
        let line = function.line_info.get(ip).copied().unwrap_or(0);
        Some((function.name.clone(), line))
    }

    /// Main interpretation loop
    pub fn interpret(&mut self) -> Result<(), RuntimeError> {
        match self.interpret_inner() {
            Ok(()) => {
                self.close_resources_owned_by(None);
                Ok(())
            }
            Err(e) => {
                self.capture_error_location();
                self.abort_all_task_scopes();
                self.close_resources_owned_by(None);
                Err(e)
            }
        }
    }
}
