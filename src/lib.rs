use axum::{routing::IntoMakeService, Extension, Router, Server};
use http::{Request, Response};
use hyper::server::conn::AddrIncoming;
use hyper::Body;
use sqlx::PgPool;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnRequest, OnResponse, TraceLayer};
use tracing::{Level, Span};

mod api;

mod settings;

pub use settings::Settings;

pub use api::owner::{
    ApiPayload, CreateOwner, CreateOwnerResponse, Owner, UpdateCredentials, UpdateProfile,
};

#[allow(unused)]
#[derive(Debug)]
pub struct AppState {
    db: PgPool,
    config: Settings,
}

#[derive(Debug, Clone)]
struct OnResponseTrace;

impl<B> OnResponse<B> for OnResponseTrace {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status_code = response.status().as_u16();

        span.record("status_code", status_code);

        span.record("latency", format_args!("{} μs", latency.as_micros()));

        tracing::event!(
            Level::INFO,
            latency = format_args!("{} μs", latency.as_micros()),
            status = status_code,
            "finished processing request"
        );
    }
}

pub fn serve(
    addr: &SocketAddr,
    db: PgPool,
    config: Settings,
) -> Server<AddrIncoming, IntoMakeService<axum::Router>> {
    let app = Router::new()
        .merge(api::health_check::router())
        .merge(api::owner::router())
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        // .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .make_span_with(|_request: &Request<Body>| {
                            tracing::info_span!(
                                "http-request",
                                status_code = tracing::field::Empty,
                                greeting = tracing::field::Empty
                            )
                        })
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(OnResponseTrace),
                )
                .layer(Extension(Arc::new(AppState { db, config }))),
        );

    axum::Server::bind(addr).serve(app.into_make_service())
}
