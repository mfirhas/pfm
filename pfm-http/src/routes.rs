use axum::{routing::get, Router};
use pfm_core::forex::interface::ForexStorage;
use tower::ServiceBuilder;

use crate::global::{self, AppContext};
use crate::middlewares;

mod forex_handlers;
mod root_handlers;

pub fn register_routes() -> Router {
    let app_ctx = global::context();
    let routes = Router::new()
        .nest("/", root_handlers())
        .nest("/forex", forex_handlers())
        .with_state(app_ctx)
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(
            middlewares::processing_time_middleware,
        )));

    routes
}

fn root_handlers<FS>() -> Router<AppContext<FS>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
{
    Router::new().route("/ping", get(root_handlers::ping::ping_handler))
}

fn forex_handlers<FS>() -> Router<AppContext<FS>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/convert", get(forex_handlers::convert::convert_handler))
        .route("/rates", get(forex_handlers::rates::get_rates_handler))
        .route(
            "/timeseries",
            get(forex_handlers::timeseries::get_timeseries_handler),
        )
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(
            crate::middlewares::api_key_middleware,
        )))
}
