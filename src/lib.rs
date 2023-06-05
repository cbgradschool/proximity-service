use axum::{routing::IntoMakeService, Extension, Router, Server};
use http::header::HeaderName;
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnRequest, TraceLayer},
};
use tracing::Level;

mod api;

mod settings;

mod telemetry;

pub use settings::Settings;

pub use api::owner::{
    ApiPayload, CreateOwner, CreateOwnerResponse, Owner, UpdateCredentials, UpdateProfile,
};

pub use telemetry::{HTTPRequestId, InitialSpan, OnResponseTrace};

#[allow(unused)]
#[derive(Debug)]
pub struct AppState {
    db: PgPool,
    config: Settings,
}

pub fn serve(
    addr: &SocketAddr,
    db: PgPool,
    config: Settings,
) -> Server<AddrIncoming, IntoMakeService<axum::Router>> {
    let x_request_id = HeaderName::from_static("x-request-id");

    let app = Router::new()
        .merge(api::health_check::router())
        .merge(api::owner::router())
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    x_request_id.clone(),
                    HTTPRequestId::default(),
                ))
                .layer(PropagateRequestIdLayer::new(x_request_id))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(InitialSpan::new())
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(OnResponseTrace),
                )
                .layer(Extension(Arc::new(AppState { db, config }))),
        );

    axum::Server::bind(addr).serve(app.into_make_service())
}
