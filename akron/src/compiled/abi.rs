use std::ffi::c_void;
use std::fmt;
use std::mem::size_of;
use std::ops::{BitOr, BitOrAssign};

use super::calls::{execute_instruction, finish_call, prepare_call, prepare_known_call};
use super::context::{
    execution_window, interpreter_bailout, load_register, poll, poll_block, poll_fast_block,
    poll_tier1_block, raise_error, refund_block, register_window, store_register,
};
use super::specializations::{list_index, list_push};

pub const RUNTIME_ABI_VERSION: u32 = 5;
pub type RuntimeStatus = u32;
pub type CompiledEntry =
    unsafe extern "C" fn(*const RuntimeApi, *mut c_void, u32, u32, *mut u64) -> RuntimeStatus;

pub const STATUS_OK: RuntimeStatus = 0;
pub const STATUS_RUNTIME_ERROR: RuntimeStatus = 1;
pub const STATUS_INVALID_ARGUMENT: RuntimeStatus = 2;
/// The compiled entry reported an internal failure without a recoverable error.
///
/// This status does not represent a recovered Rust panic. Unexpected panics are
/// fatal at the compiled runtime ABI boundary.
pub const STATUS_INTERNAL_ERROR: RuntimeStatus = 3;
pub const STATUS_BAILOUT_REQUIRED: RuntimeStatus = 4;
pub const STATUS_NATIVE_CALL_COMPLETE: RuntimeStatus = 5;
pub const STATUS_INTERPRETER_COMPLETED: RuntimeStatus = 6;
pub const STATUS_CALL_INTERPRETER_REQUIRED: RuntimeStatus = 7;
pub const STATUS_SLOW_PATH_REQUIRED: RuntimeStatus = 8;
pub const STATUS_KNOWN_CALL_MISS: RuntimeStatus = 9;
pub const STATUS_SPECIALIZATION_MISS: RuntimeStatus = 10;

pub const ERROR_INVALID_OPERAND: u32 = 1;
pub const ERROR_DIVISION_BY_ZERO: u32 = 2;
pub const ERROR_INTEGER_OVERFLOW: u32 = 3;
pub const ERROR_ASSERTION_FAILED: u32 = 4;
pub const ERROR_STACK_OVERFLOW: u32 = 5;

pub type LoadRegisterFn = unsafe extern "C" fn(*mut c_void, u32, u32, *mut u64) -> RuntimeStatus;
pub type StoreRegisterFn = unsafe extern "C" fn(*mut c_void, u32, u32, u64) -> RuntimeStatus;
pub type PollFn = unsafe extern "C" fn(*mut c_void, u32, u32) -> RuntimeStatus;
pub type RaiseErrorFn = unsafe extern "C" fn(*mut c_void, u32) -> RuntimeStatus;
pub type InterpreterBailoutFn =
    unsafe extern "C" fn(*mut c_void, u32, u32, *mut u64) -> RuntimeStatus;
pub type RegisterWindowFn =
    unsafe extern "C" fn(*mut c_void, u32, u32, *mut *mut u64) -> RuntimeStatus;
pub type PollBlockFn = unsafe extern "C" fn(*mut c_void, u32, u32, u32) -> RuntimeStatus;
pub type PollTier1BlockFn = unsafe extern "C" fn(*mut c_void, u32, u32, u32, u32) -> RuntimeStatus;
pub type PollFastBlockFn = unsafe extern "C" fn(*mut c_void, u32, u32, u32, u32) -> RuntimeStatus;
pub type RefundBlockFn = unsafe extern "C" fn(*mut c_void, u32, u32, u32) -> RuntimeStatus;
pub type ExecuteInstructionFn = unsafe extern "C" fn(*mut c_void, u32, u32) -> RuntimeStatus;
pub type PrepareCallFn =
    unsafe extern "C" fn(*mut c_void, u32, u32, *mut u32, *mut u32, *mut u32) -> RuntimeStatus;
pub type PrepareKnownCallFn =
    unsafe extern "C" fn(*mut c_void, u32, u32, u32, *mut u32, *mut u32) -> RuntimeStatus;
pub type FinishCallFn = unsafe extern "C" fn(*mut c_void, u32, u64) -> RuntimeStatus;
pub type ExecutionWindowFn = unsafe extern "C" fn(
    *mut c_void,
    u32,
    u32,
    *mut *mut u64,
    *mut *mut c_void,
    *mut u32,
) -> RuntimeStatus;
pub type SpecializeInstructionFn = unsafe extern "C" fn(*mut c_void, u32, u32) -> RuntimeStatus;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct RuntimeCapabilities(u64);

impl RuntimeCapabilities {
    pub const REGISTER_IO: Self = Self(1 << 0);
    pub const POLL: Self = Self(1 << 1);
    pub const RAISE_ERROR: Self = Self(1 << 2);
    pub const INTERPRETER_BAILOUT: Self = Self(1 << 3);
    pub const REGISTER_WINDOW: Self = Self(1 << 4);
    pub const BLOCK_POLL: Self = Self(1 << 5);
    pub const BLOCK_ACCOUNTING: Self = Self(1 << 6);
    pub const RUNTIME_INSTRUCTION: Self = Self(1 << 7);
    pub const COMPILED_CALLS: Self = Self(1 << 8);
    pub const EXECUTION_STATS: Self = Self(1 << 9);
    pub const GLOBAL_WINDOW: Self = Self(1 << 10);
    pub const FAST_POLL: Self = Self(1 << 11);
    pub const KNOWN_CALLS: Self = Self(1 << 12);
    pub const LIST_SPECIALIZATION: Self = Self(1 << 13);
    pub const CORE: Self = Self(Self::REGISTER_IO.0 | Self::POLL.0 | Self::RAISE_ERROR.0);
    pub const LLVM_BASELINE: Self = Self(Self::CORE.0 | Self::INTERPRETER_BAILOUT.0);
    pub const LLVM_TIER1: Self = Self(
        Self::LLVM_BASELINE.0
            | Self::REGISTER_WINDOW.0
            | Self::BLOCK_POLL.0
            | Self::BLOCK_ACCOUNTING.0
            | Self::RUNTIME_INSTRUCTION.0
            | Self::COMPILED_CALLS.0
            | Self::EXECUTION_STATS.0
            | Self::GLOBAL_WINDOW.0,
    );
    pub const LLVM_TIER2: Self = Self(
        Self::LLVM_TIER1.0 | Self::FAST_POLL.0 | Self::KNOWN_CALLS.0 | Self::LIST_SPECIALIZATION.0,
    );

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for RuntimeCapabilities {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RuntimeCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Append-only C ABI table passed to compiled code.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RuntimeApi {
    pub magic: [u8; 8],
    pub abi_version: u32,
    pub struct_size: u32,
    pub capabilities: RuntimeCapabilities,
    pub load_register: LoadRegisterFn,
    pub store_register: StoreRegisterFn,
    pub poll: PollFn,
    pub raise_error: RaiseErrorFn,
    pub interpreter_bailout: InterpreterBailoutFn,
    pub register_window: RegisterWindowFn,
    pub poll_block: PollBlockFn,
    pub refund_block: RefundBlockFn,
    pub execute_instruction: ExecuteInstructionFn,
    pub prepare_call: PrepareCallFn,
    pub finish_call: FinishCallFn,
    pub poll_tier1_block: PollTier1BlockFn,
    pub execution_window: ExecutionWindowFn,
    pub poll_fast_block: PollFastBlockFn,
    pub prepare_known_call: PrepareKnownCallFn,
    pub list_push: SpecializeInstructionFn,
    pub list_index: SpecializeInstructionFn,
}

#[repr(C)]
struct RuntimeApiV1 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size: u32,
    capabilities: RuntimeCapabilities,
    load_register: LoadRegisterFn,
    store_register: StoreRegisterFn,
    poll: PollFn,
    raise_error: RaiseErrorFn,
    interpreter_bailout: InterpreterBailoutFn,
}

pub const RUNTIME_ABI_V1_SIZE: u32 = size_of::<RuntimeApiV1>() as u32;

#[repr(C)]
struct RuntimeApiV2 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size: u32,
    capabilities: RuntimeCapabilities,
    load_register: LoadRegisterFn,
    store_register: StoreRegisterFn,
    poll: PollFn,
    raise_error: RaiseErrorFn,
    interpreter_bailout: InterpreterBailoutFn,
    register_window: RegisterWindowFn,
    poll_block: PollBlockFn,
    refund_block: RefundBlockFn,
    execute_instruction: ExecuteInstructionFn,
    prepare_call: PrepareCallFn,
    finish_call: FinishCallFn,
    poll_tier1_block: PollTier1BlockFn,
    execution_window: ExecutionWindowFn,
}

pub const RUNTIME_ABI_V2_SIZE: u32 = size_of::<RuntimeApiV2>() as u32;

#[repr(C)]
struct RuntimeApiV3 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size: u32,
    capabilities: RuntimeCapabilities,
    load_register: LoadRegisterFn,
    store_register: StoreRegisterFn,
    poll: PollFn,
    raise_error: RaiseErrorFn,
    interpreter_bailout: InterpreterBailoutFn,
    register_window: RegisterWindowFn,
    poll_block: PollBlockFn,
    refund_block: RefundBlockFn,
    execute_instruction: ExecuteInstructionFn,
    prepare_call: PrepareCallFn,
    finish_call: FinishCallFn,
    poll_tier1_block: PollTier1BlockFn,
    execution_window: ExecutionWindowFn,
    poll_fast_block: PollFastBlockFn,
}

pub const RUNTIME_ABI_V3_SIZE: u32 = size_of::<RuntimeApiV3>() as u32;

#[repr(C)]
struct RuntimeApiV4 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size: u32,
    capabilities: RuntimeCapabilities,
    load_register: LoadRegisterFn,
    store_register: StoreRegisterFn,
    poll: PollFn,
    raise_error: RaiseErrorFn,
    interpreter_bailout: InterpreterBailoutFn,
    register_window: RegisterWindowFn,
    poll_block: PollBlockFn,
    refund_block: RefundBlockFn,
    execute_instruction: ExecuteInstructionFn,
    prepare_call: PrepareCallFn,
    finish_call: FinishCallFn,
    poll_tier1_block: PollTier1BlockFn,
    execution_window: ExecutionWindowFn,
    poll_fast_block: PollFastBlockFn,
    prepare_known_call: PrepareKnownCallFn,
}

pub const RUNTIME_ABI_V4_SIZE: u32 = size_of::<RuntimeApiV4>() as u32;
pub const RUNTIME_ABI_V5_SIZE: u32 = size_of::<RuntimeApi>() as u32;

impl RuntimeApi {
    pub fn validate(
        &self,
        required_version: u32,
        required_size: u32,
        required_capabilities: RuntimeCapabilities,
    ) -> Result<(), RuntimeAbiError> {
        if self.magic != *b"AKRTABI\0" {
            return Err(RuntimeAbiError::InvalidMagic);
        }
        if required_version == 0 || self.abi_version < required_version {
            return Err(RuntimeAbiError::Version {
                required: required_version,
                provided: self.abi_version,
            });
        }
        if self.struct_size < required_size {
            return Err(RuntimeAbiError::TableSize {
                required: required_size,
                provided: self.struct_size,
            });
        }
        if !self.capabilities.contains(required_capabilities) {
            return Err(RuntimeAbiError::Capabilities {
                required: required_capabilities.bits(),
                provided: self.capabilities.bits(),
            });
        }
        Ok(())
    }
}

static RUNTIME_API: RuntimeApi = RuntimeApi {
    magic: *b"AKRTABI\0",
    abi_version: RUNTIME_ABI_VERSION,
    struct_size: size_of::<RuntimeApi>() as u32,
    capabilities: RuntimeCapabilities::LLVM_TIER2,
    load_register,
    store_register,
    poll,
    raise_error,
    interpreter_bailout,
    register_window,
    poll_block,
    refund_block,
    execute_instruction,
    prepare_call,
    finish_call,
    poll_tier1_block,
    execution_window,
    poll_fast_block,
    prepare_known_call,
    list_push,
    list_index,
};

pub fn runtime_api() -> &'static RuntimeApi {
    &RUNTIME_API
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAbiError {
    InvalidMagic,
    Version { required: u32, provided: u32 },
    TableSize { required: u32, provided: u32 },
    Capabilities { required: u64, provided: u64 },
}

impl fmt::Display for RuntimeAbiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "invalid runtime ABI magic"),
            Self::Version { required, provided } => write!(
                formatter,
                "runtime ABI version mismatch: required {required}, provided {provided}"
            ),
            Self::TableSize { required, provided } => write!(
                formatter,
                "runtime ABI table is too small: required {required}, provided {provided}"
            ),
            Self::Capabilities { required, provided } => write!(
                formatter,
                "runtime ABI capabilities 0x{provided:x} do not satisfy 0x{required:x}"
            ),
        }
    }
}

impl std::error::Error for RuntimeAbiError {}
