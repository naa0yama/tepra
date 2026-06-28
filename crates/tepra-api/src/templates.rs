//! Template file listing utilities.
//!
//! `list_templates` is fully implemented in T17; this stub exists so the
//! handler can reference it during the RED phase.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Metadata for a single template file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateEntry {
    /// Path relative to the template directory, using forward slashes.
    pub path: String,
}

/// Enumerate template files under `dir`.
///
/// # Errors
/// Always panics in the stub phase (RED); full implementation in T17.
#[allow(clippy::todo, clippy::module_name_repetitions)]
pub fn list_templates(
    _dir: &Path,
) -> Result<Vec<TemplateEntry>, Box<dyn std::error::Error + Send + Sync>> {
    todo!("T17")
}
