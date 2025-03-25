use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::trace::{
    Sampler, SdkTracerProvider, SpanData, SpanExporter,
    TracerProviderBuilder as SdkTracerProviderBuilder,
};

#[derive(Debug)]
pub struct NoExporter;

impl SpanExporter for NoExporter {
    fn export(
        &self,
        _batch: Vec<SpanData>,
    ) -> impl std::future::Future<
        // https://github.com/rust-lang/rust/issues/121718
        Output = std::result::Result<(), opentelemetry_sdk::error::OTelSdkError>,
    > + std::marker::Send {
        Box::pin(async { Ok(()) })
    }
}

pub struct NoExporterWrapper(pub NoExporter);

pub trait ExporterWrapper {
    fn apply_exporter(self, builder: SdkTracerProviderBuilder) -> SdkTracerProviderBuilder;
}

impl ExporterWrapper for NoExporterWrapper {
    fn apply_exporter(self, builder: SdkTracerProviderBuilder) -> SdkTracerProviderBuilder {
        builder
    }
}

impl<E> ExporterWrapper for E
where
    E: SpanExporter + Send + Sync + 'static,
{
    fn apply_exporter(self, builder: SdkTracerProviderBuilder) -> SdkTracerProviderBuilder {
        builder.with_simple_exporter(self)
    }
}

/// Builder to configure and build the trace provider.
///
/// The user could, for example, write :
///
/// ```rust
/// use otelx_core::builder::TracerProviderBuilder;
/// use opentelemetry_sdk::resource::Resource;
/// use opentelemetry_sdk::trace::Sampler;
/// use opentelemetry_stdout::SpanExporter;
///
/// let provider = TracerProviderBuilder::new()
///     .with_sampler(Sampler::AlwaysOn)
///     .with_resource(Resource::builder().with_service_name("my-api-name").build())
///     .with_exporter(SpanExporter::default())
///     .build();
///
/// opentelemetry::global::set_tracer_provider(provider);
/// ```
///
/// If no exporter is provided, the builder will use `NoExporterWrapper`.
pub struct TracerProviderBuilder<E = NoExporterWrapper> {
    sampler: Option<Sampler>,
    resource: Option<Resource>,
    exporter: E,
}

impl TracerProviderBuilder<NoExporterWrapper> {
    pub fn new() -> Self {
        Self {
            sampler: None,
            resource: None,
            exporter: NoExporterWrapper(NoExporter),
        }
    }
}

// https://rust-lang.github.io/rust-clippy/master/index.html#new_without_default
impl Default for TracerProviderBuilder<NoExporterWrapper> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: ExporterWrapper> TracerProviderBuilder<E> {
    pub fn with_sampler(mut self, sampler: Sampler) -> Self {
        self.sampler = Some(sampler);
        self
    }

    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    pub fn with_exporter<F>(self, exporter: F) -> TracerProviderBuilder<F>
    where
        F: SpanExporter + Send + Sync + 'static,
    {
        TracerProviderBuilder {
            sampler: self.sampler,
            resource: self.resource,
            exporter,
        }
    }

    pub fn build(self) -> SdkTracerProvider {
        let mut builder = SdkTracerProvider::builder();
        if let Some(sampler) = self.sampler {
            builder = builder.with_sampler(sampler);
        }
        if let Some(resource) = self.resource {
            builder = builder.with_resource(resource);
        }
        builder = self.exporter.apply_exporter(builder);
        builder.build()
    }
}
