#[cfg(feature = "axum")]
use otelx_axum::AxumTraceAdapter as ActiveTraceAdapter;

#[cfg(not(feature = "axum"))]
use crate::tracing::IdentityAdapter as ActiveTraceAdapter;

use std::future::Future;

use opentelemetry::{
    Context, KeyValue,
    global::{self, BoxedSpan},
    trace::{Span, TraceContextExt, Tracer},
};

use otelx_core::{TraceAdapter, Traceable};

/// Create a new span and return a new Context containing it.
/// This is the simple and default way to start tracing.
pub fn create_span(
    semantic: &str,
    span_name: &str,
    extra_attrs: Option<Vec<KeyValue>>,
    parent_cx: Option<&Context>,
) -> Context {
    let (span, _) = start_span(semantic, span_name, extra_attrs, parent_cx);
    Context::current_with_span(span)
}

/// Advanced usage: returns the raw span and its parent context.
/// Useful when you want to pass the parent context to another async block before propagating the new span.
pub fn start_span(
    semantic: &str,
    span_name: &str,
    extra_attrs: Option<Vec<KeyValue>>,
    parent: Option<&Context>,
) -> (BoxedSpan, Context) {
    let tracer = global::tracer("otelx.tracing.start_span");

    let parent_cx = parent.cloned().unwrap_or_else(Context::current);

    let mut span = tracer.start_with_context(span_name.to_string(), &parent_cx);

    let mut attributes = vec![
        KeyValue::new("otelx.semantic", semantic.to_string()),
        KeyValue::new("otelx.span_name", span_name.to_string()),
    ];

    if let Some(mut extra) = extra_attrs {
        attributes.append(&mut extra);
    }

    span.set_attributes(attributes);

    (span, parent_cx)
}

/// Core tracing block for types implementing `Traceable`.
/// Automatically finalizes the span and records it from the returned result.
pub async fn trace_block<T, F>(semantic: &str, span_name: &str, fut: F) -> T
where
    F: Future<Output = T>,
    T: Traceable,
{
    let cx = create_span(semantic, span_name, None, None);
    let span = cx.span();

    let result = fut.await;
    result.record_span(span);

    result
}

/// Adapter-aware tracing block.
/// Uses the active trace adapter (Axum, SQLx, etc.) selected via feature flags.
pub fn trace_with_adapter<Fut, T>(
    semantic: &str,
    span_name: &str,
    fut: Fut,
) -> impl Future<Output = T>
where
    Fut: Future<Output = T>,
    T: Traceable,
    ActiveTraceAdapter: TraceAdapter<Fut>,
    <ActiveTraceAdapter as TraceAdapter<Fut>>::Output: Future<Output = T>,
{
    let adapted = ActiveTraceAdapter::adapt(fut);
    trace_block(semantic, span_name, adapted)
}

// Default adapter for when no specific feature (like axum or sqlx) is enabled.
pub struct IdentityAdapter;

impl<Fut> TraceAdapter<Fut> for IdentityAdapter
where
    Fut: Future,
{
    type Output = Fut;

    fn adapt(fut: Fut) -> Self::Output {
        fut
    }
}
