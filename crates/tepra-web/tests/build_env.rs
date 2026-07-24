//! Tests for build-time environment variables set by build.rs.

#[test]
fn git_hash_is_populated() {
    let hash = env!("GIT_HASH");
    assert!(
        hash.len() >= 7 && hash != "unknown",
        "GIT_HASH should be a git commit hash of at least 7 chars, got: {hash:?}"
    );
}
