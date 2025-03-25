use axum::{
    body::{Body, to_bytes},
    extract::Request,
    response::Response,
};

use opentelemetry::{
    KeyValue,
    global::{self, ObjectSafeSpan},
    trace::{TraceContextExt, Tracer},
};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct OtelXMiddleware<S> {
    pub inner: S,
}

impl<S> Service<Request<Body>> for OtelXMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (mut parts, body) = req.into_parts();
        let method = parts.method.to_string();
        let uri = parts.uri.to_string();
        let version = parts.version.to_string();

        let tracer = global::tracer("otelx-axum-middleware");
        let parent_ctx =
            global::get_text_map_propagator(|prop| prop.extract(&HeaderExtractor(&parts.headers)));
        let mut span = tracer.start_with_context(format!("HTTP {method} {uri}"), &parent_ctx);

        span.set_attribute(KeyValue::new("http.method", method));
        span.set_attribute(KeyValue::new("http.target", uri));
        span.set_attribute(KeyValue::new("http.flavor", version));

        let ctx = opentelemetry::Context::current_with_span(span);

        global::get_text_map_propagator(|prop| {
            prop.inject_context(&ctx, &mut HeaderInjector(&mut parts.headers))
        });

        let req = Request::from_parts(parts, body);
        let future = self.inner.call(req);

        Box::pin(async move {
            let response = future.await?;
            let status = response.status();
            let span = ctx.span();

            span.set_attribute(KeyValue::new(
                "http.status_code",
                i64::from(status.as_u16()),
            ));

            if status.is_client_error() || status.is_server_error() {
                let (head, body) = response.into_parts();
                let body_bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
                let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                for (name, value) in &head.headers {
                    if let Ok(val_str) = value.to_str() {
                        span.set_attribute(KeyValue::new(
                            format!("http.response.header.{}", name.as_str()),
                            val_str.to_string(),
                        ));
                    }
                }

                span.set_attribute(KeyValue::new("error", true));
                span.set_attribute(KeyValue::new("error.body", body_str));

                let new_response = Response::from_parts(head, Body::from(body_bytes));

                span.end();
                Ok(new_response)
            } else {
                span.end();
                Ok(response)
            }
        })
    }
}

#[derive(Clone)]
pub struct OtelXLayer;

impl<S> Layer<S> for OtelXLayer {
    type Service = OtelXMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelXMiddleware { inner }
    }
}
