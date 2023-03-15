use proximity_service::make_server;
use std::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr = TcpListener::bind("127.0.0.1:8080")
        .expect("Failed to bind to port")
        .local_addr()
        .unwrap();

    let server = make_server(&addr);

    server.await.unwrap()
}
