mod dto;
mod global;
mod middlewares;
mod routes;

use std::sync::Arc;

use pfm_utils::{graceful_util, tracing_util};
use tokio::sync::Notify;

#[tokio::main]
async fn main() {
    tracing_util::init_tracing("pfm-http");

    // graceful shutdown
    let notify_signal = Arc::new(Notify::new());
    graceful_util::graceful_shutdown(notify_signal.clone(), Some(do_cleanup())).await;

    let routes = routes::register_routes();

    let addr = ("127.0.0.1", global::config().http_port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("httpserver: failed listening to tcp");

    tracing::info!(
        "pfm-http server is running listening on {}",
        listener.local_addr().expect("httpserver: invalid address")
    );

    axum::serve(listener, routes)
        .with_graceful_shutdown(graceful_util::wait_for_shutdown(notify_signal))
        .await
        .expect("httpserver failed");
}

/// cleanup routine to run before shutdown
async fn do_cleanup() {
    tracing::info!("cleanup start...");
    // code here...
    tracing::info!("cleanup done!")
}
