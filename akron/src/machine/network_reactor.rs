//! Bounded readiness reactor for TCP resources.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::machine::blocking_pool::JobReceiver;
use crate::native::NativeNetworkRequest;
use crate::RuntimeError;

#[cfg(not(target_arch = "wasm32"))]
mod native_impl;
#[cfg(target_arch = "wasm32")]
mod wasm_impl;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native_impl::NetworkReactor;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm_impl::NetworkReactor;
