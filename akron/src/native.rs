use crate::error::RuntimeError;
use crate::machine::VM;
use crate::specs::{CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, ResourceEffect};
use memory::{Value, ValueResourceKind};
use std::fs::File;
use std::net::SocketAddr;
use std::time::Duration;

// The unified signature for ALL extensions (Internal or External)
// args: Slice of values from the stack.
// Return: Result<Value, RuntimeError> (RuntimeError for type mismatches, etc.)
pub type NativeFn = fn(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError>;

/// Heap-independent result produced by a blocking worker and materialized back
/// into a VM [`Value`] on the owning lane.
#[derive(Debug)]
pub enum NativeAsyncOutput {
    Nil,
    Int(i64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    /// An in-VM immutable/sendable value completed by the channel hub.
    Value(Value),
    FileResource {
        handle: u32,
        file: File,
    },
    Resource {
        kind: ValueResourceKind,
        handle: u32,
    },
}

/// Owned host work safe to execute on the bounded blocking pool.
pub type NativeJob = Box<dyn FnOnce() -> Result<NativeAsyncOutput, String> + Send + 'static>;

/// A readiness-driven network operation owned by the VM's single reactor.
#[derive(Debug)]
pub enum NativeNetworkRequest {
    Connect { handle: u32, address: SocketAddr },
    Listen { handle: u32, address: SocketAddr },
    Accept { listener: u32, connection: u32 },
    Read { handle: u32, max_bytes: usize },
    Write { handle: u32, bytes: Vec<u8> },
    Close { handle: u32 },
}

/// Bounded in-VM channel operation.
#[derive(Debug)]
pub enum NativeChannelRequest {
    Send { handle: u32, value: Value },
    Receive { handle: u32 },
}

/// Host work prepared on the VM lane and dispatched without exposing VM state
/// to worker or reactor threads.
pub enum NativeAsyncWork {
    Blocking(NativeJob),
    Network(NativeNetworkRequest),
    Channel(NativeChannelRequest),
    /// Complete only after taking a FIFO turn on the VM ready queue.
    Yield,
    Timer(Duration),
}

/// Prepared async native request plus any reserved resource handles that the
/// scheduler must attach to the spawned child before dispatch.
pub struct NativeAsyncRequest {
    pub(crate) work: NativeAsyncWork,
    pub(crate) created_resources: Vec<u32>,
}

impl NativeAsyncRequest {
    pub fn blocking(job: NativeJob) -> Self {
        Self {
            work: NativeAsyncWork::Blocking(job),
            created_resources: Vec::new(),
        }
    }

    pub fn network(request: NativeNetworkRequest) -> Self {
        Self {
            work: NativeAsyncWork::Network(request),
            created_resources: Vec::new(),
        }
    }

    pub fn channel(request: NativeChannelRequest) -> Self {
        Self {
            work: NativeAsyncWork::Channel(request),
            created_resources: Vec::new(),
        }
    }

    pub fn yield_now() -> Self {
        Self {
            work: NativeAsyncWork::Yield,
            created_resources: Vec::new(),
        }
    }

    pub fn timer(duration: Duration) -> Self {
        Self {
            work: NativeAsyncWork::Timer(duration),
            created_resources: Vec::new(),
        }
    }

    pub fn with_created_resource(mut self, handle: u32) -> Self {
        self.created_resources.push(handle);
        self
    }
}

/// Prepare host work on the VM lane. Implementations copy and validate
/// arguments and authority before returning a heap-independent request.
pub type NativeAsyncStart =
    fn(vm: &mut VM, args: &[Value]) -> Result<NativeAsyncRequest, RuntimeError>;

/// Signature for prototype methods.
/// `receiver` is the value the method is called on (already type-checked by tag).
/// `args` contains only the explicit arguments (receiver is NOT included).
pub type MethodFn = fn(vm: &mut VM, receiver: Value, args: &[Value]) -> Result<Value, RuntimeError>;

#[derive(Clone)]
pub struct NativeObj {
    pub name: String,
    pub func: NativeFn,
    pub arity: isize, // -1 for variadic
    pub effects: EffectSet,
    pub capabilities: CapabilitySet,
    pub behavior: NativeBehavior,
    pub cancellation: CancellationPolicy,
    pub resource: ResourceEffect,
    pub async_start: Option<NativeAsyncStart>,
}
