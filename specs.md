# Otelx Specifications

## Overview

Otelx is a Rust library for integrating OpenTelemetry in a uniform and explicit way. It centralizes configuration, context propagation, and span creation for asynchronous, event-driven applications—without relying on implicit globals.

## Architecture

- Explicit Configuration

  - **ConfigBuilder**: A fluent API to define resources (e.g., service name), sampling strategy, semantic keys, and a custom exporter.
  - **Config**: The finalized configuration passed explicitly to all components.

- Context & Propagation

  - **Context**: A structure encapsulating trace context (trace ID, span ID, attributes).
  - **Span**: Provides methods to add attributes, events, set status, and end the span.
  - **create_span(semantic, value, parent)**: Creates a span using a semantic key (defined via a trait) with an optional parent context.
  - **with_context(context, f)**: Wraps async tasks to explicitly propagate the trace context.

- Exporter

  - **Exporter Trait**: Defines an async method to export spans.
  - Default implementations are provided, and users can implement custom exporters.

- Semantics
  - **Semantic Trait**: Requires a method `to_key_value()` to convert a semantic key to an OpenTelemetry `KeyValue`.
  - **Semantic Enum**: Centralized, strongly-typed definition (e.g., `SERVICE`, `ERROR_SOURCE`, etc.) that can also wrap constants from opentelemetry-semantic-conventions.

Merci pour ta patience et tes précisions. Afin de bien cerner ton besoin, je vais te proposer une spécification détaillée du système **`otelx`**, en mettant l'accent sur la simplicité d'intégration et l'unification des traces et spans dans un projet complet.

### **Objectif du Système `otelx`**

L'objectif de **`otelx`** est de fournir une **interface simple, cohérente et robuste** pour intégrer facilement des traces OpenTelemetry dans un projet, sans nécessiter de manipulations complexes des contextes et spans. L'utilisateur doit pouvoir instrumenter son code de manière intuitive et bénéficier d'une traçabilité riche et complète (incluant les paramètres, erreurs, requêtes SQL, réponses HTTP, etc.), sans devoir gérer manuellement les contextes ou les spans.

### **Principaux Objectifs à Atteindre :**

1. **Simplification** : L'utilisateur ne doit pas avoir à gérer manuellement les contextes et les spans.
2. **Cohérence** : Tous les spans doivent suivre une nomenclature cohérente et enrichie automatiquement.
3. **Informations Complètes** : Les traces doivent inclure toutes les informations pertinentes par défaut (ex. paramètres de la fonction, résultats des requêtes SQL, code de statut HTTP, messages d'erreur, etc.).
4. **Facilité d'intégration** : L'utilisateur doit pouvoir instrumenter rapidement ses fonctions avec une simple annotation ou un appel d'une fonction.
5. **Adaptabilité** : Le système doit être extensible pour ajouter facilement des intégrations avec des outils comme **SQLx**, **Axum**, ou d'autres technologies via des features activables dans le `Cargo.toml`.

### **Composants Clés de `otelx`**

1. **`otelx-core`** : Le cœur du système qui gère la logique de création des spans, l'enregistrement des paramètres, et la propagation des traces. Il contient les abstractions de base.
2. **`otelx-attributes`** : Un module qui contient une macro d'attribut pour automatiser l'instrumentation des fonctions.
3. **`otelx-axum`** : Intégration avec Axum pour traquer les réponses HTTP.
4. **`otelx-sqlx`** : Intégration avec SQLx pour traquer les requêtes SQL.
5. **`otelx`** : Point d'entrée principal qui expose toutes les fonctionnalités et permet d'activer les intégrations via des **features**.

### **Spécification du Système `otelx`**

---

#### 1. **`otelx-core`** : La Logique de Base

Ce module contient la logique principale de création de spans et de gestion des traces.

##### **Fonctions principales :**

- **`create_span`** :

  - Crée un span avec des attributs de base : `otelx.semantic` et `otelx.span_name`.
  - Enregistre le contexte dans un `Context` qui peut être propagé à d'autres parties du système.

  ```rust
  pub fn create_span(semantic: &str, span_name: &str) -> Context {
      let tracer = global::tracer("otelx.tracing.create_span");
      let mut span = tracer.start(span_name.to_string());
      span.set_attribute(KeyValue::new("otelx.semantic", semantic.to_string()));
      span.set_attribute(KeyValue::new("otelx.span_name", span_name.to_string()));
      Context::current_with_span(span)
  }
  ```

- **`trace_block`** :

  - Exécute un bloc de code asynchrone tout en enregistrant un span pour ce bloc. À la fin de l'exécution, le span est finalisé automatiquement.
  - Il est générique pour permettre l'enregistrement d'un span autour de n'importe quelle future.

  ```rust
  pub async fn trace_block<T, F>(semantic: &str, span_name: &str, fut: F) -> T
  where
      F: Future<Output = T>,
  {
      let cx = create_span(semantic, span_name);
      let span = cx.span();
      let result = fut.await;
      span.end();
      result
  }
  ```

- **Enregistrement automatique des paramètres** :

  - Lors de l'exécution d'une fonction instrumentée, les paramètres de la fonction sont automatiquement ajoutés aux spans en tant qu'attributs.

  Cela se fait via l'utilisation de la macro **`#[otelx::tracing]`** (voir section suivante).

---

#### 2. **`otelx-attributes`** : La Macro d'Instrumentation

La macro `#[otelx::tracing]` permet à l'utilisateur de tracer facilement une fonction en ajoutant un span automatiquement autour de celle-ci.

##### **Fonctionnement de la macro** :

- **Paramètres** :
  - `semantic` : Le nom sémantique de l'opération (par exemple, `HttpGetUserTokens`).
  - `span_name` : Le nom du span (par exemple, `HTTP GET /user/tokens`).
- **Ajout automatique des paramètres de fonction au span** :

  - Les paramètres de la fonction sont automatiquement récupérés et ajoutés au span sous forme de paires clé-valeur.

  Exemple :

  ```rust
  #[otelx::tracing(semantic = "HttpGetUserTokens", span_name = "HTTP GET /user/tokens")]
  pub async fn get_user_tokens(
      State(api_state): State<ApiState>,
      Extension(user_id): Extension<Uuid>,
  ) -> axum::response::Response {
      // Corps de la fonction
  }
  ```

  Ce code va automatiquement créer un span pour la fonction, y inclure les paramètres (`api_state`, `user_id`), et ensuite envoyer la trace à OpenTelemetry sans que l'utilisateur ait à gérer les spans ou contextes manuellement.

---

#### 3. **`otelx-axum`** : Intégration avec Axum

Ce module permet d'intégrer la traçabilité des réponses HTTP d'Axum.

- **Adaptateur `AxumResponseAdapter`** :

  - Capture les informations de statut HTTP, ajoute des attributs au span et gère les erreurs HTTP de manière cohérente.

  Exemple :

  ```rust
  pub struct AxumResponseAdapter<T>(pub Response<T>);

  impl<T> Traceable for AxumResponseAdapter<T> {
      fn record_span(&self, span: SpanRef<'_>) {
          let status = self.0.status().as_u16();
          span.set_attribute(KeyValue::new("http.status_code", status as i64));
          if status >= 500 {
              span.set_status(Status::error("internal server error"));
          } else {
              span.set_status(Status::Ok);
          }
          span.end();
      }
  }
  ```

  Les utilisateurs n'ont qu'à utiliser l'adaptateur pour avoir des traces cohérentes des réponses HTTP.

---

#### 4. **`otelx-sqlx`** : Intégration avec SQLx

Ce module trace les requêtes SQL exécutées avec **SQLx**.

- **Adaptateur `SqlxQueryAdapter`** :

  - Capture le texte de la requête SQL, les erreurs et ajoute des attributs à un span.

  Exemple :

  ```rust
  pub struct SqlxQueryAdapter<'a, T> {
      pub query: &'a str,
      pub result: Result<Vec<T>, SqlxError>,
  }

  impl<'a, T> Traceable for SqlxQueryAdapter<'a, T>
  where
      T: std::fmt::Debug,
  {
      fn record_span(&self, span: SpanRef<'_>) {
          span.set_attribute(KeyValue::new("sql.query", self.query.to_string()));
          match &self.result {
              Ok(_) => span.set_status(Status::Ok),
              Err(e) => {
                  span.set_status(Status::error(format!("{:?}", e)));
                  span.set_attribute(KeyValue::new("sql.error", format!("{:?}", e)));
              }
          }
          span.end();
      }
  }
  ```

---

#### 5. **Simplification et Consistance** :

- **Création et propagation des spans** : L'utilisateur n'a qu'à appliquer la macro `#[otelx::tracing]` sur ses fonctions pour que les spans soient automatiquement créés et propagés.
- **Nommage cohérent** : Les noms des spans sont générés de manière cohérente et logique en utilisant les paramètres `semantic` et `span_name`.
- **Enrichissement automatique** : Les paramètres de fonction, les résultats des requêtes SQL, et les réponses HTTP sont automatiquement ajoutés aux spans en tant qu'attributs.
- **Pas besoin de gérer les contextes** : Tout est géré automatiquement par **`otelx`**, il n'est pas nécessaire de manipuler des `Context` ou des spans manuellement.

---

### **Exemple d'Utilisation Complet**

Voici un exemple d'application **`otelx`** dans une fonction Axum avec SQLx :

```rust
#[otelx::tracing(semantic = "HttpGetUserTokens", span_name = "HTTP GET /user/tokens")]
pub async fn get_user_tokens(
    State(api_state): State<ApiState>,
    Extension(user_id): Extension<Uuid>,
) -> axum::response::Response {
    let tokens = match sqlx::query_as::<_, TokenToken>(
        "SELECT t.id, t.token FROM tokens t JOIN j_user_token jut ON t.id = jut.token_id WHERE jut.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&api_state.db_pool)
    .await
    {
        Ok(tokens) => tokens,
        Err(sqlx::Error::RowNotFound) => {
            return response_err(StatusCode::BAD_REQUEST, "No tokens found".to_string());
        }
        Err(err) => {
            error!("Cannot retrieve tokens, err = {}", err);
            return response_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string());
        }
    };

    response_success(StatusCode::OK, json!({ "tokens": tokens }))
}
```

---

### **Résumé**

- **Simplicité d'usage** : Avec `otelx`, l'utilisateur ajoute une simple annotation pour tracer ses fonctions.
- **Traçabilité complète** : Les spans sont créés automatiquement, les paramètres de fonction, résultats SQL, et codes de statut HTTP sont enrichis sans effort.
- **Consistance et Cohérence** : Les spans suivent un modèle de nommage cohérent, et les informations critiques sont capturées automatiquement, garantissant une traçabilité fiable à travers toute l'application.

L'objectif est de permettre aux utilisateurs de se concentrer sur la logique de leur application, tout en bénéficiant d'une instrumentations complète et cohérente des traces.
