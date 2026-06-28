//! Template file listing utilities.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

/// Metadata for a single template file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateEntry {
    /// Path relative to the template directory, using forward slashes.
    pub path: String,
}

/// Enumerate `.lbl` template files under `dir` recursively.
///
/// Relative paths in the returned entries always use forward slashes so that
/// they are safe to embed in JSON responses on any OS (Windows `\` → `/`).
///
/// # Errors
/// Returns an error if `dir` does not exist or cannot be read.
#[allow(clippy::module_name_repetitions)]
pub fn list_templates(dir: &Path) -> anyhow::Result<Vec<TemplateEntry>> {
    anyhow::ensure!(
        dir.exists(),
        "template directory not found: {}",
        dir.display()
    );

    let mut entries = Vec::new();
    walk(dir, dir, &mut entries)?;
    Ok(entries)
}

fn walk(root: &Path, current: &Path, out: &mut Vec<TemplateEntry>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(current)
        .with_context(|| format!("failed to read directory: {}", current.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read dir entry in: {}", current.display()))?;
        let path: PathBuf = entry.path();
        let ft = entry
            .file_type()
            .with_context(|| format!("failed to get file type: {}", path.display()))?;
        if ft.is_dir() {
            walk(root, &path, out)?;
        } else if ft.is_file() && is_lbl(&path) {
            let rel = path
                .strip_prefix(root)
                .with_context(|| format!("failed to strip prefix from: {}", path.display()))?;
            // Normalize path separators to forward slash (Windows compat).
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            out.push(TemplateEntry { path: rel_str });
        }
    }
    Ok(())
}

fn is_lbl(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lbl"))
}
