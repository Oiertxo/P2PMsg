use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use libp2p::PeerId;

pub fn init_p2p_logging(storage_path: &str, peer_id: &PeerId) -> WorkerGuard {
    let log_dir = Path::new(storage_path).join("logs");

    // Create log folder
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("CRITICAL: Error while creating log folder: {:?}", e);
    }

    // Log config
    let file_name = format!("{}.log", peer_id.to_string());
    let file_appender = tracing_appender::rolling::never(log_dir, file_name);
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Log format
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into())
            .add_directive("libp2p_kad=warn".parse().unwrap()))
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Initializing Logger for Peer: {}", peer_id);

    guard
}
