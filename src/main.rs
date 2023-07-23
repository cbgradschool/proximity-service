use opentelemetry::{
    global::shutdown_tracer_provider,
    metrics, runtime,
    sdk::{
        export::metrics::aggregation::cumulative_temporality_selector,
        metrics::{controllers::BasicController, selectors},
        trace::{self, RandomIdGenerator},
        Resource,
    },
    trace::TraceError,
    Context, KeyValue,
};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use proximity_service::{serve, Settings};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tonic::metadata::{MetadataMap, MetadataValue};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, registry::Registry, EnvFilter};

use url::Url;

fn init_tracer(config: &Settings) -> Result<opentelemetry::sdk::trace::Tracer, TraceError> {
    let mut metadata = MetadataMap::with_capacity(3);

    metadata.insert_bin(
        "trace-proto-bin",
        MetadataValue::from_bytes(b"[binary data]"),
    );

    let host_and_port = format!(
        "{}:{}",
        config.otel_collector_host, config.otel_collector_port
    );

    let endpoint = Url::parse(&host_and_port).expect("endpoint is not a valid url");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(metadata)
                .with_timeout(std::time::Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "proximity-service",
                )])),
        )
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

fn init_metrics(config: &Settings) -> metrics::Result<BasicController> {
    let host_and_port = format!("{}:{}", config.honeycomb_host, config.honeycomb_port);

    let endpoint = Url::parse(&host_and_port).expect("endpoint is not a valid url");

    let export_config = ExportConfig {
        endpoint: endpoint.as_str().to_string(),
        ..ExportConfig::default()
    };

    opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            cumulative_temporality_selector(),
            runtime::TokioCurrentThread,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .build()
}

async fn handle_graceful_shutdown(metrics_controller: BasicController, ctx: Context) {
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigint.recv() => {
            info!("SIGINT signal received, shutting down...");
        }
        _ = sigterm.recv() => {
            info!("SIGTERM signal received, shutting down...");
        }
    }

    metrics_controller.stop(&ctx).unwrap();

    shutdown_tracer_provider();
}

#[tokio::main]
async fn main() {
    let config = Settings::new().unwrap();

    let tracer = init_tracer(&config).unwrap();

    let metrics_controller = init_metrics(&config).unwrap();

    let ctx = Context::new();

    metrics_controller
        .start(&ctx, runtime::TokioCurrentThread)
        .unwrap();

    let subscriber = Registry::default()
        .with(
            // OpenTelemetry layer
            tracing_opentelemetry::layer().with_tracer(tracer),
        )
        .with(EnvFilter::new("INFO"))
        .with(
            // Pretty print layer
            fmt::layer()
                .pretty()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .unwrap();

    let addr = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .unwrap_or_else(|_| panic!("Failed to bind to PORT:{}", config.port))
        .local_addr()
        .unwrap();

    let server = serve(&addr, db, config)
        .with_graceful_shutdown(handle_graceful_shutdown(metrics_controller, ctx));

    info!("Starting up proximity_service...");

    if let Err(e) = server.await {
        error!("proximity_service failed to start: {:?}", e)
    }
}
