use axum::{routing::IntoMakeService, Extension, Router, Server};
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

mod api;

mod settings;

pub use settings::Settings;

pub use api::owner::{ApiPayload, CreateOwner, CreateOwnerResponse, Owner};

pub struct AppState {
    db: PgPool,
    config: Settings
}

pub fn serve(addr: &SocketAddr, db: PgPool, config: Settings) -> Server<AddrIncoming, IntoMakeService<axum::Router>> {
    let app = Router::new()
        .merge(api::health_check::router())
        .merge(api::owner::router())
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                )
                .layer(Extension(Arc::new(AppState { db, config }))),
        );

    axum::Server::bind(addr).serve(app.into_make_service())
}
