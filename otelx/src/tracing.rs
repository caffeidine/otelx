use opentelemetry::Context;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{
    KeyValue, global,
    trace::{Span, Tracer},
};
use std::future::Future;

/// Crée un span simple et renvoie un contexte avec ce span.
pub fn create_span(semantic: &str, span_name: &str) -> Context {
    let tracer = global::tracer("otelx.tracing.create_span");
    let mut span = tracer.start(span_name.to_string());

    span.set_attribute(KeyValue::new("otelx.semantic", semantic.to_string()));
    span.set_attribute(KeyValue::new("otelx.span_name", span_name.to_string()));

    Context::current_with_span(span)
}

/// Exécute une tâche asynchrone et enregistre un span pour cette tâche.
pub async fn trace_block<T, F>(semantic: &str, span_name: &str, fut: F) -> T
where
    F: Future<Output = T>,
{
    let cx = create_span(semantic, span_name);
    let span = cx.span();
    let result = fut.await;
    span.end();
    result
}
