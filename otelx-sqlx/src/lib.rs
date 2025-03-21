use opentelemetry::trace::SpanRef;

pub struct PgQueryResultWrapper(pub sqlx::postgres::PgQueryResult);

impl otelx_core::Traceable for PgQueryResultWrapper {
    fn record_span(&self, span: opentelemetry::trace::SpanRef<'_>) {
        span.set_attribute(opentelemetry::KeyValue::new(
            "rows_affected",
            self.0.rows_affected() as i64,
        ));
    }
}

#[derive(Debug)]
pub struct SqlxErrorWrapper(pub sqlx::Error);

impl otelx_core::Traceable for SqlxErrorWrapper {
    fn record_span(&self, span: SpanRef<'_>) {
        span.set_attribute(opentelemetry::KeyValue::new(
            "sqlx_error",
            format!("{:?}", self),
        ));
    }
}

pub fn wrap_pg_query_result(res: sqlx::postgres::PgQueryResult) -> PgQueryResultWrapper {
    PgQueryResultWrapper(res)
}
