mod dto;
mod global;
mod middlewares;
mod routes;

#[tokio::main]
async fn main() {
    let routes = routes::register_routes();

    let addr = ("127.0.0.1", global::config().http_port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("httpserver: failed listening to tcp");

    println!(
        "server is listening on {}",
        listener.local_addr().expect("httpserver: invalid address")
    );

    axum::serve(listener, routes)
        .await
        .expect("httpserver failed");
}
