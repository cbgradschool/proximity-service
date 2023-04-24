use dotenvy::dotenv;
use std::env;
use std::net::TcpListener;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn spawn_app() -> (String, Pool<Postgres>) {
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

    let settings = proximity_service::Settings::new().unwrap();

    let server = proximity_service::serve(&addr, db.clone(), settings);

    tokio::spawn(server);

    (format!("http://127.0.0.1:{}", port), db)
}

#[allow(dead_code)] // bug: https://github.com/rust-lang/rust/issues/46379
pub async fn test_setup(db: &Pool<Postgres>) -> i32 {
    /*! Seeds database with a single "Owner" record */

    sqlx::query_scalar!(
        r#"insert into "owners" (name, email, password) values ($1, $2, $3) returning id;"#,
        "David Hayter",
        "solidsnake@sonsofliberty.com",
        "lalilulelo"
    )
    .fetch_one(db)
    .await
    .unwrap()
}

#[allow(dead_code)] // bug: https://github.com/rust-lang/rust/issues/46379
pub async fn test_teardown(id: i32, db: &Pool<Postgres>) -> Result<(), ()> {
    let _delete = sqlx::query_scalar!(r#"delete from "owners" where id = $1"#, id,)
        .fetch_one(db)
        .await;

    Ok(())
}
