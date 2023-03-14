
use proximity_service::make_server;

#[tokio::main]
async fn main() {
    let server = make_server();

    server.await.unwrap()
}
