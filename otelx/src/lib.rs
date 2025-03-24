pub use otelx_core::*;

#[cfg(feature = "axum")]
pub use otelx_axum::*;

#[cfg(feature = "sqlx")]
pub use otelx_sqlx::*;

pub use otelx_attributes::tracing;


use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{span, Event, Id, Metadata, Subscriber};
use tracing_core::span::{Attributes, Record};
use tracing_core::field::Visit;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
struct MinimalSubscriber;

impl Subscriber for MinimalSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        // Ici, vous pouvez filtrer les spans/événements selon leur niveau ou autre critère
        true
    }

    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        // Générez un identifiant unique pour le span
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed) as u64;
        Id::from_u64(id)
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {
        // Enregistrez les mises à jour des valeurs d’un span, si nécessaire
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        // Ici, vous pouvez, par exemple, collecter les données de l’événement
        // via un Visitor pour ensuite les stocker dans une base de données
        println!("Événement: {} - {}", event.metadata().level(), event.metadata().name());
        // Vous pourriez utiliser un visitor personnalisé pour extraire les champs
    }

    fn enter(&self, span: &Id) {
        println!("Entrée du span {:?}", span);
    }

    fn exit(&self, span: &Id) {
        println!("Sortie du span {:?}", span);
    }

    // D'autres méthodes fournies par défaut peuvent rester inchangées
}
 
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Installez votre subscriber personnalisé comme subscriber global
    tracing::subscriber::set_global_default(MinimalSubscriber)
        .expect("Impossible d'installer le subscriber");
 
    // Testez avec un span et un événement
    let my_span = span!(tracing::Level::INFO, "mon_span", champ = 42);
    let _enter = my_span.enter();
    tracing::info!("Ceci est un événement");
    Ok(())
}

