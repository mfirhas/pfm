use axum::{http::StatusCode, response::IntoResponse};

pub(crate) async fn ping_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
