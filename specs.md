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
