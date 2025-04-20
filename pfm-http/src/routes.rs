use axum::{routing::get, Router};
use pfm_core::forex::interface::ForexStorage;
use tower::ServiceBuilder;

use crate::global::{self, AppContext};
use crate::middlewares;

mod forex_routes;
mod root_routes;

pub fn register_routes() -> Router {
    Router::new()
        .nest("/", root_routes())
        .nest("/forex", forex_routes())
        .with_state(global::context())
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(
            middlewares::processing_time_middleware,
        )))
        .layer(axum::middleware::from_fn(middlewares::tracing_middleware))
}

fn root_routes<FS>() -> Router<AppContext<FS>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
{
    Router::new().route("/ping", get(root_routes::ping::ping_handler))
}

fn forex_routes<FS>() -> Router<AppContext<FS>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/convert", get(forex_routes::convert::convert_handler))
        .route("/rates", get(forex_routes::rates::get_rates_handler))
        .route(
            "/timeseries",
            get(forex_routes::timeseries::get_timeseries_handler),
        )
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(
            middlewares::request_id_middleware,
        )))
        .layer(
            ServiceBuilder::new().layer(axum::middleware::from_fn(middlewares::api_key_middleware)),
        )
}
