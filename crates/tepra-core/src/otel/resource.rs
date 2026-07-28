use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::attribute::{
    HOST_ARCH, HOST_NAME, OS_TYPE, PROCESS_EXECUTABLE_NAME, PROCESS_PID, PROCESS_RUNTIME_NAME,
    PROCESS_RUNTIME_VERSION, SERVICE_INSTANCE_ID, SERVICE_NAME, SERVICE_VERSION,
    VCS_REF_HEAD_REVISION,
};

/// Build an [`Resource`] with standard service attributes.
///
/// `service_name` is injected by the binary crate (e.g. `env!("CARGO_PKG_NAME")` in
/// `tepra-web`) so that `service.name` reflects the running binary, not this library.
/// `git_hash` is injected by the caller (typically `env!("GIT_HASH")`) to avoid
/// duplicating `build.rs` across crates.
#[must_use]
pub fn build(service_name: &'static str, git_hash: &'static str) -> Resource {
    let resolved_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| String::from(service_name));
    let hostname = gethostname::gethostname().to_string_lossy().into_owned();
    let pid = i64::from(std::process::id());
    let instance_id = format!("{hostname}-{pid}");

    Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, resolved_name),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(VCS_REF_HEAD_REVISION, git_hash),
            KeyValue::new(SERVICE_INSTANCE_ID, instance_id),
            KeyValue::new(HOST_NAME, hostname),
            KeyValue::new(HOST_ARCH, std::env::consts::ARCH),
            KeyValue::new(OS_TYPE, std::env::consts::OS),
            KeyValue::new(PROCESS_PID, pid),
            KeyValue::new(PROCESS_EXECUTABLE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(PROCESS_RUNTIME_NAME, "rustc"),
            KeyValue::new(PROCESS_RUNTIME_VERSION, env!("RUSTC_VERSION")),
        ])
        .build()
}

// Resource::builder() enables OTel SDK OS detectors that call libc::uname(),
// which Miri flags as uninit-memory UB in libc::utsname; skip under miri.
#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    use opentelemetry::Value;
    use serial_test::serial;

    fn get_attr(resource: &Resource, key: &str) -> Option<Value> {
        resource
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .map(|(_, v)| v.clone())
    }

    #[test]
    #[serial]
    fn build_has_service_name() {
        // SAFETY: single-threaded via #[serial]; no concurrent env access
        unsafe { std::env::remove_var("OTEL_SERVICE_NAME") };
        let r = build("tepra-web", "testhash1234");
        assert_eq!(
            get_attr(&r, SERVICE_NAME),
            Some(Value::String("tepra-web".into())),
        );
    }

    #[test]
    #[serial]
    fn build_uses_otel_service_name_env_when_set() {
        // SAFETY: single-threaded via #[serial]; no concurrent env access
        unsafe { std::env::set_var("OTEL_SERVICE_NAME", "custom-svc") };
        let r = build("tepra-web", "testhash1234");
        // SAFETY: single-threaded via #[serial]; no concurrent env access
        unsafe { std::env::remove_var("OTEL_SERVICE_NAME") };
        assert_eq!(
            get_attr(&r, SERVICE_NAME),
            Some(Value::String("custom-svc".into())),
        );
    }

    #[test]
    #[serial]
    fn build_falls_back_to_default_when_env_unset() {
        // SAFETY: single-threaded via #[serial]; no concurrent env access
        unsafe { std::env::remove_var("OTEL_SERVICE_NAME") };
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

    #[test]
    fn build_has_host_name() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "host.name").expect("host.name missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_process_pid() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "process.pid").expect("process.pid missing");
        assert!(matches!(val, Value::I64(n) if n > 0));
    }

    #[test]
    fn build_has_process_runtime_name() {
        let r = build("tepra-web", "testhash1234");
        assert_eq!(
            get_attr(&r, "process.runtime.name"),
            Some(Value::String("rustc".into())),
        );
    }

    #[test]
    fn build_has_service_instance_id() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "service.instance.id").expect("service.instance.id missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_host_arch() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "host.arch").expect("host.arch missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_os_type() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "os.type").expect("os.type missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_process_executable_name() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "process.executable.name").expect("process.executable.name missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }

    #[test]
    fn build_has_process_runtime_version() {
        let r = build("tepra-web", "testhash1234");
        let val = get_attr(&r, "process.runtime.version").expect("process.runtime.version missing");
        assert!(matches!(val, Value::String(s) if !s.as_str().is_empty()));
    }
}
