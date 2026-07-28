//! Per-printer actor queue (Phase 3).

pub mod job;
pub mod printer;
pub mod registry;

#[allow(clippy::module_name_repetitions)]
pub use printer::PrinterActor;
#[allow(clippy::module_name_repetitions)]
pub use printer::PrinterHandle;
#[allow(clippy::module_name_repetitions)]
pub use registry::PrinterRegistry;
