pub mod tracing;

pub use opentelemetry::global::BoxedSpan;
pub use opentelemetry::trace::Span;
pub use opentelemetry::{Context, KeyValue};

pub use tracing::{trace_block, trace_with_adapter};

pub use otelx_core::{TraceAdapter, Traceable};

pub use otelx_attributes::tracing;
