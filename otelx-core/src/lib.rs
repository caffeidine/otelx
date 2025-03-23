use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use opentelemetry::Context;
use opentelemetry::trace::{Span, SpanRef, TraceContextExt, Tracer as _};

/// Trait for types that can record span information
pub trait Traceable {
    fn record_span(&self, span: SpanRef<'_>);
}

/// Trait that adapts a Future before tracing.
pub trait TraceWrapper<Fut: Future> {
    type Output: Future<Output = Fut::Output>;

    fn adapt(fut: Fut) -> Self::Output;
}

/// Default implementation that just passes through the future
pub struct DefaultTraceWrapper;

impl<Fut> TraceWrapper<Fut> for DefaultTraceWrapper
where
    Fut: Future + 'static,
{
    type Output = Pin<Box<dyn Future<Output = Fut::Output>>>;

    fn adapt(fut: Fut) -> Self::Output {
        Box::pin(fut)
    }
}

/// Trace an async operation with a given semantic key and span name.
/// This function is primarily used by the `#[otelx::tracing]` attribute macro.
pub async fn trace_with_adapter<F, Fut, T, W>(
    semantic: &'static str,
    span_name: &'static str,
    fut: F,
    params: HashMap<&'static str, String>,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
    W: TraceWrapper<Fut>,
    T: Traceable,
{
    let tracer = opentelemetry::global::tracer("otelx");
    let mut span = tracer
        .span_builder(span_name)
        .with_attributes(vec![opentelemetry::KeyValue::new(
            "otelx.semantic",
            semantic,
        )])
        .start(&tracer);

    // Add all parameters as span attributes
    for (key, value) in params {
        span.set_attributes(vec![opentelemetry::KeyValue::new(key, value)]);
    }

    let cx = Context::current_with_value(span);
    let result = W::adapt(fut()).await;
    result.record_span(cx.span());
    result
}

/// Wrapper for types that don't need span recording
pub struct NoSpanRecording<T>(pub T);

impl<T> Traceable for NoSpanRecording<T> {
    fn record_span(&self, _span: SpanRef<'_>) {}
}
