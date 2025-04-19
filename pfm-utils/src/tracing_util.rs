use tracing::info;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

#[cfg(feature = "otel")]
use opentelemetry::sdk::trace as sdktrace;
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracing(service_name: &'static str) {
    let is_release = cfg!(not(debug_assertions));
    let log_level = if is_release { "info" } else { "debug" };

    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        // .with_thread_names(true)
        // .with_thread_ids(true)
        // .json(); // Optional: switch to .pretty() for human-readable
        .pretty();

    #[cfg(feature = "otel")]
    {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
            .with_trace_config(
                sdktrace::config().with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name),
                ])),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to install OTLP pipeline");

        let otel_layer = OpenTelemetryLayer::new(tracer);

        Registry::default()
            .with(filter_layer)
            .with(fmt_layer)
            .with(otel_layer)
            .init();
    }

    #[cfg(not(feature = "otel"))]
    {
        Registry::default()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }

    info!("Tracing initialized");
}
