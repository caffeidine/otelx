#[macro_export]
macro_rules! otel_event {
    // Syntax: otel_event!(level, “message”, key1 = value1, key2 = value2, ...);
    ($level:expr, $msg:expr $(, $key:ident = $value:expr )* $(,)?) => {{
        use opentelemetry::KeyValue;
        let mut attrs = Vec::new();
        $(
            attrs.push(KeyValue::new(stringify!($key), $value.to_string()));
        )*
        // Adds the level as an attribute
        attrs.push(KeyValue::new("otel.level", $level.to_string()));

        $crate::add_event($msg, Some(attrs));
    }};
}

#[macro_export]
macro_rules! otel_span {
    // Syntaxe simple : otel_span!(level, "span_name", { bloc de code });
    ($level:expr, $span_name:expr, { $($body:tt)* } $(,)?) => {{
        use opentelemetry::KeyValue;

        let (ctx, span) = $crate::new_child_span(
            $span_name,
            Some(vec![KeyValue::new("otel.level", $level.to_string())])
        );

        let result = { $($body)* };
        span.end();
        result
    }};
    // Syntax with attributes: otel_span!(level, “span_name”, { code block }, key1 = value1, key2 = value2, ...);
    ($level:expr, $span_name:expr, { $($body:tt)* }, $( $key:ident = $value:expr ),+ $(,)?) => {{
        use opentelemetry::KeyValue;
        let mut attrs = Vec::new();
        $(
            attrs.push(KeyValue::new(stringify!($key), $value.to_string()));
        )+

        attrs.push(KeyValue::new("otel.level", $level.to_string()));
        let (ctx, span) = $crate::new_child_span($span_name, Some(attrs));
        let result = { $($body)* };
        span.end();
        result
    }};
}
