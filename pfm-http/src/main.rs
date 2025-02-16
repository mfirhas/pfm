use axum::{
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use pfm_core::utils::get_config;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let cfg = get_config::<Config>("HTTP_").expect("failed getting config");

    let routes = Router::new().route("/ping", get(ping));

    let addr = ("127.0.0.1", cfg.http_port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("httpserver: failed listening to tcp");

    println!(
        "server is listening on {}",
        listener.local_addr().expect("httpserver: invalid address")
    );

    let routes_group = Router::new().nest("/", routes);

    axum::serve(listener, routes_group)
        .await
        .expect("httpserver failed");
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    #[serde(alias = "HTTP_PORT")]
    pub http_port: u16,
}

async fn ping() -> impl IntoResponse {
    "pong"
}
