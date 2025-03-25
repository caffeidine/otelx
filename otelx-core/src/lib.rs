pub mod builder;
pub mod macros;

use opentelemetry::{
    Context, KeyValue,
    trace::{Span, TraceContextExt, Tracer},
};

pub fn current_context() -> Context {
    Context::current()
}

pub fn add_event(message: &str, attributes: Option<Vec<KeyValue>>) {
    let ctx = current_context();
    let span = ctx.span();
    match attributes {
        Some(attrs) => span.add_event(message.to_string(), attrs),
        None => span.add_event(message.to_string(), vec![]),
    }
}

pub fn new_child_span(name: &str, attributes: Option<Vec<KeyValue>>) -> Context {
    let parent_ctx = current_context();
    let tracer = opentelemetry::global::tracer("child");
    let mut span = tracer.start_with_context(name.to_string(), &parent_ctx);
    if let Some(attrs) = attributes {
        span.set_attributes(attrs);
    }
    Context::current_with_span(span)
}

pub fn set_context(ctx: Context) -> opentelemetry::ContextGuard {
    ctx.attach()
}

pub use opentelemetry::KeyValue as OtelXKeyValue;
