use std::{future::Future, sync::Arc};

use tokio::{signal, sync::Notify};

/// Spawning concurrent graceful shutdown routine triggered by signal listened through notify_shutdown.
/// notify_shutdown notify the graceful coroutine
/// let notify_shutdown = Arc::new(Notify::new());
pub async fn graceful_shutdown<F>(notify_signal: Arc<Notify>, cleanup_fn: Option<F>)
where
    F: Future<Output = ()> + Send + Sync + 'static,
{
    // Spawn a background task to listen for shutdown signal
    tokio::spawn(async move {
        // Wait for Ctrl+C or SIGTERM
        if let Err(e) = shutdown_signal().await {
            tracing::error!("shutdown signal error: {:?}", e);
        }
        tracing::info!("💀💀💀 Shutdown signal received.");

        // Perform any pre-shutdown async logic here
        if let Some(cleanup_func) = cleanup_fn {
            tracing::info!("Doing clean up...");
            cleanup_func.await;
        }

        // Notify the server to shut down
        notify_signal.notify_one();
    });
}

/// Wait for shutdown notification
pub async fn wait_for_shutdown(notify: Arc<Notify>) {
    notify.notified().await;
}

/// Handles OS signals (Ctrl+C or SIGTERM)
async fn shutdown_signal() -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate())?;
        let ctrl_c = signal::ctrl_c();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Signal ctrl+c received...")
            },
            _ = sigterm.recv() => {
                tracing::info!("Signal sigterm received...")
            },
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c().await?;
    }

    Ok(())
}
