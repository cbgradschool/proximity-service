use dotenvy::dotenv;
use std::env;
use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

pub async fn spawn_app() -> String {
    dotenv().ok();

    let db_connection_url = env::var("DATABASE_URL").expect("Database connection url not found.");

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_connection_url)
        .await
        .unwrap();
    let addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap();

    let port = addr.port();

    let server = proximity_service::serve(&addr, db);

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
