use axum::response::Response;
use otelx_core::{TraceWrapper, Traceable};
use std::{future::Future, pin::Pin};

pub struct AxumResponseAdaptater<T>(pub Response<T>);

impl<T> AxumResponseAdaptater<T> {
    pub fn inner(&self) -> &Response<T> {
        &self.0
    }
}

impl<T> Traceable for AxumResponseAdaptater<T> {
    fn record_span(&self, span: opentelemetry::trace::SpanRef<'_>) {
        let status = self.0.status().as_u16();
        span.set_attribute(opentelemetry::KeyValue::new(
            "http.status_code",
            status as i64,
        ));
        if status >= 500 {
            span.set_status(opentelemetry::trace::Status::error("internal server error"));
        } else {
            span.set_status(opentelemetry::trace::Status::Ok);
        }
        span.end();
    }
}

impl<Fut, T> TraceWrapper<Fut> for AxumResponseAdaptater<T>
where
    Fut: Future<Output = Response<T>> + Send + 'static,
    T: Send + 'static,
{
    type Output = Pin<Box<dyn Future<Output = AxumResponseAdaptater<T>>>>;

    fn adapt(fut: Fut) -> Self::Output {
        Box::pin(async move {
            let resp = fut.await;
            AxumResponseAdaptater(resp)
        })
    }
}
