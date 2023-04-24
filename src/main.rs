use proximity_service::{serve, Settings};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match Settings::new() {
        Ok(config) => {
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

            info!("proximity_service server starting up...");

            server.await.unwrap()
        }
        Err(err) => {
            warn!(
                "proximity_service has not been properly configured. Error: {}",
                err
            );
        }
    }
}
