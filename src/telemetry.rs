use http::{HeaderName, HeaderValue, Request, Response};
use std::{fmt, time::Duration};
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{MakeSpan, OnFailure, OnResponse},
};
use tracing::{Level, Span};
use uuid::Uuid;

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
