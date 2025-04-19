mod dto;
mod global;
mod middlewares;
mod routes;

use pfm_utils::tracing_util;

#[tokio::main]
async fn main() {
    tracing_util::init_tracing("pfm-http");

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
        .await
        .expect("httpserver failed");
}
