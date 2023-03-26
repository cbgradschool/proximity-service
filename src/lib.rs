use axum::{
    routing::IntoMakeService,
    Router, Server, Extension,
};
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};

mod api;

pub use api::owner::ApiPayload;
pub use api::owner::CreateOwner;

pub struct AppState {
    db: PgPool
}

pub fn serve(addr: &SocketAddr, db: PgPool) -> Server<AddrIncoming, IntoMakeService<axum::Router>> {

    let app = Router::new()
        .merge(api::health_check::router())
        .merge(api::owner::router())
        .layer(Extension(Arc::new(AppState{db})));

    axum::Server::bind(addr).serve(app.into_make_service())
}
