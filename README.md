# otelx

`otelx` is a crate designed to provide a unified and simplified interface for using OpenTelemetry and the OpenTelemetry SDK in Rust.

## Features

- **Simple Initialization**: A single function to configure OpenTelemetry with sensible defaults.
- **Basic Context Management**: A minimal API to manage and propagate telemetry context.
- **Custom Exporter Interface**: A trait that allows users to implement their own asynchronous exporters.

## Getting Started

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
otelx = "0.0.1"
```

For more details on the design and features, please refer to the [specs.md](specs.md) file.
