use crate::Span;

pub trait SpanFinalizer {
    fn finalize_span(span: &Span, result: &Self);
}

impl<T> SpanFinalizer for T {
    default fn finalize_span(_span: &Span, _result: &Self) {}
}

pub fn finalize_span<T: SpanFinalizer>(span: &Span, result: &T) {
    T::finalize_span(span, result);
}
