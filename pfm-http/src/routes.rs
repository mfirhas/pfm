use axum::{routing::get, Router};
use pfm_core::forex::interface::{ForexHistoricalRates, ForexStorage};
// use tower::ServiceBuilder;

use crate::global::{self, AppContext};
use crate::middlewares;

mod admin_routes;
mod forex_routes;
mod root_routes;

pub fn register_routes() -> Router {
    Router::new()
        .nest("/", root_routes())
        .nest("/admin", admin_routes())
        .nest("/forex", forex_routes())
        .with_state(global::context())
        .layer(axum::middleware::from_fn(
            middlewares::processing_time_middleware,
        ))
        .layer(axum::middleware::from_fn(middlewares::tracing_middleware))
}

// ---------------- ROUTES ----------------

fn root_routes<FS, FH>() -> Router<AppContext<FS, FH>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
    FH: ForexHistoricalRates + Clone + Send + Sync + 'static,
{
    Router::new().route("/ping", get(root_routes::ping::ping_handler))
}

pub fn admin_routes<FS, FH>() -> Router<AppContext<FS, FH>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
    FH: ForexHistoricalRates + Clone + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/forex/fetch_historical_rates",
            get(admin_routes::historical_rates::fetch_historical_rates_handler),
        )
        .layer(axum::middleware::from_fn(
            middlewares::admin_password_middleware,
        ))
}

fn forex_routes<FS, FH>() -> Router<AppContext<FS, FH>>
where
    FS: ForexStorage + Clone + Send + Sync + 'static,
    FH: ForexHistoricalRates + Clone + Send + Sync + 'static,
{
    let routes = Router::new()
        .route("/convert", get(forex_routes::convert::convert_handler))
        .route("/rates", get(forex_routes::rates::get_rates_handler))
        .route(
            "/timeseries",
            get(forex_routes::timeseries::get_timeseries_handler),
        );

    if global::config().enable_api_key {
        return routes.layer(axum::middleware::from_fn(middlewares::api_key_middleware));
    }

    routes
}
