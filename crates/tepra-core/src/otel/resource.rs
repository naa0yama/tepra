use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::attribute::{
    SERVICE_NAME, SERVICE_VERSION, VCS_REF_HEAD_REVISION,
};

/// Build an [`Resource`] with standard service attributes.
///
/// `service_name` is injected by the binary crate (e.g. `env!("CARGO_PKG_NAME")` in
/// `tepra-web`) so that `service.name` reflects the running binary, not this library.
/// `git_hash` is injected by the caller (typically `env!("GIT_HASH")`) to avoid
/// duplicating `build.rs` across crates.
#[must_use]
pub fn build(service_name: &'static str, git_hash: &'static str) -> Resource {
    Resource::builder_empty()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, service_name),
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
        let r = build("tepra-web", "testhash1234");
        assert_eq!(
            get_attr(&r, SERVICE_NAME),
            Some(Value::String("tepra-web".into())),
        );
    }

    #[test]
    fn build_has_service_version() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, SERVICE_VERSION).expect("service.version missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_git_hash() {
        let r = build("tepra-web", "testhash1234");
        assert_eq!(
            get_attr(&r, VCS_REF_HEAD_REVISION),
            Some(Value::String("testhash1234".into())),
        );
    }
}
