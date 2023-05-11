use opentelemetry::{global::shutdown_tracer_provider, sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_otlp::WithExportConfig;
use proximity_service::{serve, Settings};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, net::TcpListener};
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, registry::Registry};

fn init_tracer(config: &Settings) -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(format!(
                    "{}/v1/traces:{}",
                    config.honeycomb_host, config.honeycomb_port
                ))
                .with_http_client(reqwest::Client::default())
                .with_headers(HashMap::from([
                    (
                        "x-honeycomb-dataset".into(),
                        config.honeycomb_dataset.clone(),
                    ),
                    ("x-honeycomb-team".into(), config.honeycomb_api_key.clone()),
                ]))
                .with_timeout(std::time::Duration::from_secs(2)),
        ) // Replace with runtime::Tokio if using async main
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

async fn handle_shutdown_signal() {
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigint.recv() => {
            info!("Interrupt signal received, shutting down...");
        }
        _ = sigterm.recv() => {
            info!("Terminate signal received, shutting down...");
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

    let server = serve(&addr, db, config);

    tokio::spawn(async move {
        info!("proximity_service starting up...");

        server.await.unwrap();
    });

    // Handle shutdown gracefully
    handle_shutdown_signal().await;
}
