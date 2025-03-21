use std::future::Future;

use opentelemetry::trace::SpanRef;

pub trait Traceable {
    fn record_span(&self, span: SpanRef<'_>);
}

/// Trait that adapts a Future before tracing.
pub trait TraceAdapter<Fut> {
    type Output: Future;

    fn adapt(fut: Fut) -> Self::Output;
}
