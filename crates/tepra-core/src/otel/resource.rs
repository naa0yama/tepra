use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::attribute::{
    SERVICE_NAME, SERVICE_VERSION, VCS_REF_HEAD_REVISION,
};

/// Build an [`Resource`] with standard service attributes.
///
/// `git_hash` is injected by the caller (typically `env!("GIT_HASH")` in the
/// binary crate) to avoid duplicating `build.rs` across crates.
#[must_use]
pub fn build(git_hash: &'static str) -> Resource {
    Resource::builder_empty()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(VCS_REF_HEAD_REVISION, git_hash),
        ])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::Value;

    fn get_attr(resource: &Resource, key: &str) -> Option<Value> {
        resource
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .map(|(_, v)| v.clone())
    }

    #[test]
    fn build_has_service_name() {
        let r = build("testhash1234");
        assert_eq!(
            get_attr(&r, SERVICE_NAME),
            Some(Value::String("tepra-core".into())),
        );
    }

    #[test]
    fn build_has_service_version() {
        let r = build("testhash1234");
        let val = get_attr(&r, SERVICE_VERSION).expect("service.version missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_git_hash() {
        let r = build("testhash1234");
        assert_eq!(
            get_attr(&r, VCS_REF_HEAD_REVISION),
            Some(Value::String("testhash1234".into())),
        );
    }
}
