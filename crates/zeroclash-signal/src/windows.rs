use std::sync::atomic::{AtomicBool, Ordering};

use zeroclash_logging::{logging, Type};

use crate::RUNTIME;

static IS_CLEANING_UP: AtomicBool = AtomicBool::new(false);

pub fn register<F, Fut>(f: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future + Send + 'static,
{
    if let Some(Some(rt)) = RUNTIME.get() {
        rt.spawn(async move {
            let mut ctrl_c = match tokio::signal::windows::ctrl_c() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+C: {}", e);
                    return;
                }
            };

            let mut ctrl_close = match tokio::signal::windows::ctrl_close() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+Close: {}", e);
                    return;
                }
            };

            let mut ctrl_shutdown = match tokio::signal::windows::ctrl_shutdown() {
                Ok(s) => s,
                Err(e) => {
                    logging!(
                        error,
                        Type::SystemSignal,
                        "Failed to register Ctrl+Shutdown: {}",
                        e
                    );
                    return;
                }
            };

            let mut ctrl_logoff = match tokio::signal::windows::ctrl_logoff() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+Logoff: {}", e);
                    return;
                }
            };

            loop {
                let signal_name;
                tokio::select! {
                    _ = ctrl_c.recv() => {
                        signal_name = "Ctrl+C";
                    }
                    _ = ctrl_close.recv() => {
                        signal_name = "Ctrl+Close";
                    }
                    _ = ctrl_shutdown.recv() => {
                        signal_name = "Ctrl+Shutdown";
                    }
                    _ = ctrl_logoff.recv() => {
                        signal_name = "Ctrl+Logoff";
                    }
                }

                if IS_CLEANING_UP.load(Ordering::SeqCst) {
                    logging!(
                        info,
                        Type::SystemSignal,
                        "Already shutting down, ignoring repeated signal: {}",
                        signal_name
                    );
                    continue;
                }
                IS_CLEANING_UP.store(true, Ordering::SeqCst);

                logging!(
                    info,
                    Type::SystemSignal,
                    "Caught Windows signal: {}",
                    signal_name
                );

                f().await;
            }
        });
    } else {
        logging!(
            error,
            Type::SystemSignal,
            "register shutdown signal failed, RUNTIME is not available"
        );
    }
}
