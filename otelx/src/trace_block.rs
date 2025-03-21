use crate::{SpanFinalizer, create_span};
use std::future::Future;

pub async fn trace_block<T, F>(semantic: &str, span_name: &str, fut: F) -> T
where
    F: Future<Output = T>,
    T: SpanFinalizer,
{
    let (cx, span) = create_span(semantic, span_name, None, None);

    let result = fut.await;

    T::finalize_span(&span, &result);
    span.end();

    result
}
