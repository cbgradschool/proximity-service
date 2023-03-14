fn spawn_app() {
    let server = proximity_service::make_server();

    tokio::spawn(server);
}

#[tokio::test]
async fn test_health_check() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    assert_eq!(Some(0), response.content_length());
}

