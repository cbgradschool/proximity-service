use opentelemetry::{
    global::shutdown_tracer_provider,
    sdk::{
        trace::{self, RandomIdGenerator},
        Resource,
    },
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use proximity_service::{serve, Settings};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, registry::Registry, EnvFilter};

use url::Url;

fn init_tracer(config: &Settings) -> Result<opentelemetry::sdk::trace::Tracer, TraceError> {
    let mut metadata = MetadataMap::with_capacity(2);

    metadata.insert(
        "x-honeycomb-team",
        config.honeycomb_api_key.parse().unwrap(),
    );
    metadata.insert(
        "x-honeycomb-dataset",
        config.honeycomb_dataset.parse().unwrap(),
    );

    let host_and_port = format!("{}:{}", config.honeycomb_host, config.honeycomb_port);

    let endpoint = Url::parse(&host_and_port).expect("endpoint is not a valid url");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(metadata)
                .with_tls_config(
                    ClientTlsConfig::new().domain_name(
                        endpoint
                            .host_str()
                            .expect("the specified endpoint should have a valid host"),
                    ),
                )
                .with_timeout(std::time::Duration::from_secs(2)),
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

async fn handle_shutdown_signal() {
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

    shutdown_tracer_provider();
}

#[tokio::main]
async fn main() {
    let config = Settings::new().unwrap();

    let tracer = init_tracer(&config).unwrap();

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

    let server = serve(&addr, db, config).with_graceful_shutdown(handle_shutdown_signal());

    info!("Starting up proximity_service...");

    if let Err(e) = server.await {
        error!("proximity_service failed to start: {:?}", e)
    }
}
