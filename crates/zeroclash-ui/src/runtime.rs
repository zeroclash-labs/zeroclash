#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Tokio runtime ownership.
//!
//! GPUI runs the app event loop on the main thread; our async work
//! (`reqwest`, `tokio::fs`, `tokio::process`, future `tokio::spawn` calls
//! from `zeroclash-core::connection::spawn_connection_stream` /
//! `zeroclash-core::enhance::use_script`) all need a long-lived multi-thread
//! tokio runtime. We:
//!
//! 1. Build the runtime once at startup, leak it so it lives for the entire
//!    program (no chance of an accidental drop while a spawned task is still
//!    running).
//! 2. Stash a `Handle` clone in a `OnceLock` so any thread (UI render,
//!    background poll loops, listeners) can call [`handle()`] without
//!    relying on the implicit thread-local `Runtime::enter` guard.
//! 3. Enter the runtime on the calling (main) thread so `pollster::block_on`
//!    style code that ends up `.await`-ing on tokio primitives keeps working.

use std::sync::OnceLock;

use tokio::runtime::{Handle, Runtime};

static HANDLE: OnceLock<Handle> = OnceLock::new();

/// Initialize the global tokio runtime. Must be called exactly once at app
/// startup before any code attempts to spawn or poll futures backed by tokio.
///
/// Returns a tokio `EnterGuard` bound to the calling thread. Callers should
/// hold it for the lifetime of the main thread's event loop.
pub fn init() -> tokio::runtime::EnterGuard<'static> {
    let rt: &'static Runtime = Box::leak(Box::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("zeroclash-rt")
            .build()
            .expect("build tokio runtime"),
    ));
    HANDLE
        .set(rt.handle().clone())
        .expect("runtime handle already initialised");
    rt.enter()
}

/// Returns a clone of the global tokio runtime handle. Panics if [`init`]
/// has not been called.
pub fn handle() -> Handle {
    HANDLE
        .get()
        .cloned()
        .expect("zeroclash runtime not initialised; call runtime::init() at startup")
}
