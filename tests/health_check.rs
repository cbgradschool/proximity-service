use sqlx::PgPool;

mod utils;

#[sqlx::test]
async fn test_health_check(db: PgPool) {
    let (address, _) = utils::make_server(db).await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    assert_eq!(Some(0), response.content_length());
}
