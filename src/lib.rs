use axum::{
    routing::{get, IntoMakeService},
    Router, Server,
};
use hyper::server::conn::AddrIncoming;
use std::net::SocketAddr;

mod routes;

pub fn make_server(addr: &SocketAddr) -> Server<AddrIncoming, IntoMakeService<axum::Router>> {
    let app = Router::new().route("/health_check", get(routes::get_health_check));

    axum::Server::bind(addr).serve(app.into_make_service())
}
