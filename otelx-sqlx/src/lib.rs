use std::fmt;
use tracing::{Event, Level, Span, span};
use tracing_subscriber::{Layer, layer::SubscriberExt, registry::LookupSpan};

/// --- 2. La fonction trace_query() ---
/// À appeler dans `.instrument(otelx::trace_query())`.
/// Ne fournit aucune valeur par défaut ; le span sert uniquement de repère pour notre couche.
pub fn trace_query() -> Span {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .finish()
        .with(SqlxQueryLayer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
    // Crée un span avec un champ de marquage, ici "otelx_enabled" = true
    span!(Level::INFO, "otelx.sqlx.query", otelx_enabled = true)
}

/// --- 3. Couche personnalisée pour intercepter les événements SQLx ---
pub struct SqlxQueryLayer;

impl<S> Layer<S> for SqlxQueryLayer
where
    S: tracing::Subscriber,
    for<'lookup> S: LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // On cible uniquement les événements émis avec le target "sqlx::query"
        if event.metadata().target() == "sqlx::query" {
            println!("==> SQLx Event: {}", event.metadata().name());
            let mut visitor = FieldVisitor::default();
            event.record(&mut visitor);
            println!("   db.statement : {:?}", visitor.db_statement);
            println!("   rows_returned: {:?}", visitor.rows_returned);
            println!("   rows_affected: {:?}", visitor.rows_affected);
            println!("   elapsed_secs : {:?}", visitor.elapsed_secs);
        }
    }
}

/// Visitor pour extraire les champs des événements SQLx.
#[derive(Default)]
struct FieldVisitor {
    pub db_statement: Option<String>,
    pub rows_returned: Option<u64>,
    pub rows_affected: Option<u64>,
    pub elapsed_secs: Option<f64>,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        match field.name() {
            "db.statement" => self.db_statement = Some(format!("{:?}", value)),
            "rows_returned" => {
                let s = format!("{:?}", value);
                self.rows_returned = s.parse().ok();
            }
            "rows_affected" => {
                let s = format!("{:?}", value);
                self.rows_affected = s.parse().ok();
            }
            "elapsed_secs" => {
                let s = format!("{:?}", value);
                self.elapsed_secs = s.parse().ok();
            }
            _ => {}
        }
    }
}
