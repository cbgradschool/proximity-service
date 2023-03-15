use std::net::TcpListener;

pub fn spawn_app() -> String {
    let addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap();

    let port = addr.port();

    let server = proximity_service::make_server(&addr);

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
