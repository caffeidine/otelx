use axum::response::Response;
use otelx_core::{TraceWrapper, Traceable};
use std::{future::Future, pin::Pin};

/// Wrapper for axum responses that adds tracing capabilities
pub struct TracedResponse<T>(pub Response<T>);

impl<T> From<Response<T>> for TracedResponse<T> {
    fn from(response: Response<T>) -> Self {
        TracedResponse(response)
    }
}

impl<T> From<TracedResponse<T>> for Response<T> {
    fn from(traced: TracedResponse<T>) -> Self {
        traced.0
    }
}

impl<T> Traceable for TracedResponse<T> {
    fn record_span(&self, span: opentelemetry::trace::SpanRef<'_>) {
        let status = self.0.status().as_u16();
        span.set_attributes(vec![opentelemetry::KeyValue::new(
            "http.status_code",
            status as i64,
        )]);
        if status >= 500 {
            span.set_status(opentelemetry::trace::Status::error("internal server error"));
        } else {
            span.set_status(opentelemetry::trace::Status::Ok);
        }
        span.end();
    }
}

/// Wrapper for axum responses that adds tracing capabilities
pub struct AxumResponseWrapper;

impl<Fut, T> otelx_core::TraceWrapper<Fut> for AxumResponseWrapper
where
    Fut: Future<Output = Response<T>> + Send + 'static,
    T: Send + 'static,
{
    type Output = Pin<Box<dyn Future<Output = TracedResponse<T>>>>;

    fn adapt(fut: Fut) -> Self::Output {
        Box::pin(async move {
            let resp = fut.await;
            TracedResponse(resp)
        })
    }
}
