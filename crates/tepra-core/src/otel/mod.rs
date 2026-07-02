//! OpenTelemetry instrumentation root module.
//!
//! Add `tracing` / `logs` submodules here when adopting those signals.

pub mod metrics;
/// Resource builder (service identity attributes).
pub mod resource;
