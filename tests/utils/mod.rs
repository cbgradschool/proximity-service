use dotenvy::dotenv;
use std::net::TcpListener;

use sqlx::{PgPool, Pool, Postgres};

pub async fn make_server(db: PgPool) -> (String, Pool<Postgres>) {
    dotenv().ok();

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
