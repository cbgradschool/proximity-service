use proximity_service::serve;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_connection_url = env::var("DATABASE_URL").expect("Database connection url not found.");

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_connection_url)
        .await
        .unwrap();

    let addr = TcpListener::bind("127.0.0.1:8080")
        .expect("Failed to bind to port")
        .local_addr()
        .unwrap();

    let server = serve(&addr, db);

    server.await.unwrap()
}
