use http::{HeaderName, HeaderValue, Request, Response};
use std::{fmt, time::Duration};
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{MakeSpan, OnFailure, OnResponse},
};
use tracing::{Level, Span};
use uuid::Uuid;

use crate::Settings;
use opentelemetry::{
    global::shutdown_tracer_provider,
    sdk::{
        trace::{self, RandomIdGenerator},
        Resource,
    },
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use url::Url;

#[derive(Debug, Clone)]
pub struct OnResponseTrace;

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

#[derive(Debug, Clone)]
pub struct OnFailureTrace;

impl<E> OnFailure<E> for OnFailureTrace
where
    E: fmt::Display,
{
    fn on_failure(&mut self, _error: E, _latency: Duration, _span: &Span) {
        tracing::debug!("something went wrong")
    }
}

//https://betterprogramming.pub/production-grade-logging-in-rust-applications-2c7fffd108a6
#[derive(Debug, Clone)]
pub struct InitialSpan {}

impl InitialSpan {
    pub fn new() -> Self {
        Self {}
    }
}

impl<B> MakeSpan<B> for InitialSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(&HeaderName::from_static("x-request-id"))
            .unwrap();

        tracing::span!(
            Level::INFO,
            "http-request",
            status_code = tracing::field::Empty,
            method = ?request.method(),
            uri = ?request.uri(),
            version = ?request.version(),
            headers = ?request.headers(),
            request_id = ?request_id,
        )
    }
}

#[derive(Clone, Default)]
pub struct HTTPRequestId {}

impl MakeRequestId for HTTPRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4();

        let header_value = HeaderValue::from_str(&request_id.to_string()).ok()?;

        Some(RequestId::new(header_value))
    }
}

fn init_tracer(config: &Settings) -> Result<opentelemetry::sdk::trace::Tracer, TraceError> {
    let mut metadata = MetadataMap::with_capacity(2);

    metadata.insert(
        "x-honeycomb-team",
        config.honeycomb_api_key.parse().unwrap(),
    );
    metadata.insert(
        "x-honeycomb-dataset",
        config.honeycomb_dataset.parse().unwrap(),
    );

    let host_and_port = format!("{}:{}", config.honeycomb_host, config.honeycomb_port);

    let endpoint = Url::parse(&host_and_port).expect("endpoint is not a valid url");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(metadata)
                .with_tls_config(
                    ClientTlsConfig::new().domain_name(
                        endpoint
                            .host_str()
                            .expect("the specified endpoint should have a valid host"),
                    ),
                )
                .with_timeout(std::time::Duration::from_secs(2)),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "proximity-service",
                )])),
        )
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}
