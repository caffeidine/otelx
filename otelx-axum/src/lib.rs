use axum::response::Response;
use opentelemetry::KeyValue;
use opentelemetry::trace::{SpanRef, Status};
use otelx_core::{TraceAdapter, Traceable};
use std::future::Future;

pub trait AxumTraceable {
    fn record_axum_span(&self, span: SpanRef<'_>);
}

impl<T> AxumTraceable for Response<T> {
    fn record_axum_span(&self, span: SpanRef<'_>) {
        let status = self.status().as_u16();
        span.set_attribute(KeyValue::new("http.status_code", status as i64));
        if self.status().is_server_error() {
            span.set_status(Status::error("internal server error"));
        } else {
            span.set_status(Status::Ok);
        }
        span.end();
    }
}

pub struct AxumResponse<T>(pub Response<T>);

impl<T> Traceable for AxumResponse<T> {
    fn record_span(&self, span: SpanRef<'_>) {
        self.0.record_axum_span(span);
    }
}

impl<T> From<AxumResponse<T>> for Response<T> {
    fn from(wrapper: AxumResponse<T>) -> Self {
        wrapper.0
    }
}

pub struct AxumTraceAdapter;

pub struct AdaptedAxumFuture<F, T> {
    inner: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> Future for AdaptedAxumFuture<F, T>
where
    F: Future<Output = AxumResponse<T>>,
{
    type Output = AxumResponse<T>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let fut = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        fut.poll(cx)
    }
}

impl<F, T> TraceAdapter<F> for AxumTraceAdapter
where
    F: Future<Output = AxumResponse<T>>,
{
    type Output = AdaptedAxumFuture<F, T>;

    fn adapt(fut: F) -> Self::Output {
        AdaptedAxumFuture {
            inner: fut,
            _phantom: std::marker::PhantomData,
        }
    }
}
