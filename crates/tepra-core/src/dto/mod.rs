//! Data Transfer Objects for the KING JIM TEPRA Creator `WebAPI`.
//!
//! Structs in this module represent the raw JSON shapes sent to and received
//! from `http://localhost:29108/api/printer/*`. Field names and enum wire
//! values are derived from `tepraprint.js` (the authoritative source).

// DTO fields are self-documenting through their names and module-level docs.
#![allow(missing_docs, clippy::module_name_repetitions)]

pub mod enums;
pub mod error;
pub mod job;
pub mod printer;
pub mod template;
