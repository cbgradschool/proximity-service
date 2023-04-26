use opentelemetry::{sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_otlp::WithExportConfig;
use proximity_service::{serve, Settings};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, net::TcpListener};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;

fn init_tracer(config: Settings) -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(format!("{}/v1/traces", config.honeycomb_host))
                .with_http_client(reqwest::Client::default())
                .with_headers(HashMap::from([
                    ("x-honeycomb-dataset".into(), config.honeycomb_dataset),
                    ("x-honeycomb-team".into(), config.honeycomb_api_key),
                ]))
                .with_timeout(std::time::Duration::from_secs(2)),
        ) // Replace with runtime::Tokio if using async main
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

#[tokio::main]
async fn main() {
    let config = Settings::new().unwrap();

    let tracer = init_tracer(config.clone()).unwrap();

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = tracing_subscriber::Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .unwrap();

    let addr = TcpListener::bind("127.0.0.1:8080")
        .expect("Failed to bind to port")
        .local_addr()
        .unwrap();

    let server = serve(&addr, db, config);

    server.await.unwrap();

    info!("proximity_service server starting up...");

    // shutdown_tracer_provider();
}
