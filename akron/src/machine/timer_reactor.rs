//! Bounded timer service: one native thread per VM, never one per sleep.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use crate::machine::blocking_pool::JobReceiver;
use crate::RuntimeError;

#[cfg(not(target_arch = "wasm32"))]
mod native_impl;
#[cfg(target_arch = "wasm32")]
mod wasm_impl;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native_impl::TimerReactor;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm_impl::TimerReactor;
