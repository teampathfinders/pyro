#![allow(dead_code)]
#![allow(unused)]
#![allow(clippy::module_inception)]
#![warn(clippy::nursery)]

use std::sync::atomic::{AtomicU16, Ordering};

use tokio::runtime;

use crate::instance::ServerInstance;

mod config;
mod crypto;
mod error;
mod instance;
mod network;
mod util;

#[cfg(test)]
mod test;

/// Default Minecraft IPv4 port
const IPV4_PORT: u16 = 19132;
/// Default Minecraft IPv6 port
const IPV6_PORT: u16 = 19133;

/// The asynchronous entrypoint that is ran by Tokio.
async fn app_main() -> anyhow::Result<()> {
    loop {
        let controller = ServerInstance::new(IPV4_PORT, 100).await?;
        match controller.run().await {
            Ok(_) => {
                break;
            }
            Err(e) => {
                tracing::error!("The server probably crashed, restarting it...");
                tracing::error!("Cause: {e:?}");
            }
        }
    }

    Ok(())
}

/// Program entrypoint
fn main() -> anyhow::Result<()> {
    init_logging();

    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static ATOMIC_THREAD_COUNTER: AtomicU16 = AtomicU16::new(0);
            format!(
                "async-thread-{}",
                ATOMIC_THREAD_COUNTER.fetch_add(1, Ordering::Relaxed)
            )
        })
        .build()
        .expect("Failed to build runtime");

    runtime.block_on(app_main())
}

/// Initialises logging with tokio-console.
#[cfg(feature = "tokio-console")]
fn init_logging() {
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let console_layer = console_subscriber::Builder::default()
        .retention(Duration::from_secs(1))
        .recording_path("console_trace.log")
        .spawn();

    let fmt = tracing_subscriber::fmt::layer().with_target(false);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt)
        .init();

    tracing::info!("Tokio console enabled");
}

/// Initialises logging without tokio-console.
#[cfg(not(feature = "tokio-console"))]
fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .init();
}
