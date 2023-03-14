use std::net::TcpListener;

fn spawn_app() -> String {
    let addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap();

    let port = addr.port();

    let server = proximity_service::make_server(&addr);

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn test_health_check() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    assert_eq!(Some(0), response.content_length());
}
