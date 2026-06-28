//! RED unit tests for `list_templates`.
//!
//! These tests describe the intended behaviour of the full implementation
//! (T17b).  They all fail in the current stub because `list_templates`
//! calls `todo!()` and panics.
// tempfile::tempdir() calls mkdir which miri isolation blocks;
// filesystem integration tests are not the target of UB detection.
#![cfg(not(miri))]
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::items_after_statements
)]

use std::{fs, path::Path};

use tepra_api::templates::list_templates;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Create `path` (relative to `base`) including any intermediate dirs.
fn touch(base: &Path, rel: &str) {
    let full = base.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, b"").unwrap();
}

// ---------------------------------------------------------------------------
// 1. Recursive enumeration
// ---------------------------------------------------------------------------

#[test]
fn test_list_templates_recursive() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "sub1/foo.lbl");
    touch(dir.path(), "sub2/sub3/bar.lbl");

    let entries = list_templates(dir.path()).unwrap();
    assert_eq!(entries.len(), 2, "expected 2 .lbl files found recursively");

    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    assert!(
        paths.contains(&"sub1/foo.lbl"),
        "sub1/foo.lbl missing: {paths:?}"
    );
    assert!(
        paths.contains(&"sub2/sub3/bar.lbl"),
        "sub2/sub3/bar.lbl missing: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// 2. Extension filter — only .lbl files, not .txt / .md
// ---------------------------------------------------------------------------

#[test]
fn test_list_templates_extension_filter() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "label.lbl");
    touch(dir.path(), "readme.md");
    touch(dir.path(), "notes.txt");
    touch(dir.path(), "sub/other.lbl");

    let entries = list_templates(dir.path()).unwrap();
    assert_eq!(entries.len(), 2, "only .lbl files should be returned");

    for e in &entries {
        let is_lbl = Path::new(&e.path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lbl"));
        assert!(is_lbl, "non-.lbl entry returned: {}", e.path);
    }
}

// ---------------------------------------------------------------------------
// 3. Relative path uses forward slashes (Windows normalization)
// ---------------------------------------------------------------------------

#[test]
fn test_list_templates_slash_normalization() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "group_a/template.lbl");

    let entries = list_templates(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);

    let path = &entries.first().unwrap().path;
    assert!(
        !path.contains('\\'),
        "path must use forward slashes, got: {path}"
    );
    assert_eq!(path, "group_a/template.lbl");
}

// ---------------------------------------------------------------------------
// 4a. Missing directory → Err
// ---------------------------------------------------------------------------

#[test]
fn test_list_templates_missing_dir_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist");

    let result = list_templates(&missing);
    assert!(result.is_err(), "expected Err for missing directory");
}

// ---------------------------------------------------------------------------
// 4b. Empty directory → Ok(vec![])
// ---------------------------------------------------------------------------

#[test]
fn test_list_templates_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();

    let entries = list_templates(dir.path()).unwrap();
    assert!(entries.is_empty(), "expected empty vec for empty directory");
}
