pub use forsl_trace::{
    DispatchMeta, MetaGuard, TestCaptureLayer, TraceDumpLayer, current_correlation_id,
    current_meta, current_or_new_correlation_uuid, current_policy, format_ui_target_1,
    format_ui_target_2, in_named_scope, in_ui_action_scope, install_current_meta, install_policy,
    is_message_enabled, is_scope_enabled, is_target_enabled, normalize_policy, register_scopes,
};

use forsl_trace::TracePolicy;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::writer::MakeWriter;

include!(concat!(env!("OUT_DIR"), "/trace_scopes.rs"));

pub fn install_defaults() {
    register_scopes(ALL_SCOPES);
}

pub fn builtin_policy() -> TracePolicy {
    normalize_policy(TracePolicy {
        enabled_prefixes: BUILTIN_ENABLE_SCOPES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        disabled_prefixes: Vec::new(),
        disabled_message_prefixes: BUILTIN_DISABLE_MESSAGES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        disabled_target_prefixes: BUILTIN_DISABLE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        dump_capacity: TracePolicy::default().dump_capacity,
    })
}

pub fn load_policy(path: &Path) -> TracePolicy {
    forsl_trace::load_policy(path, builtin_policy())
}

pub fn init_subscriber<W>(settings_path: &Path, writer: W) -> anyhow::Result<()>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    install_defaults();

    let trace_policy = load_policy(settings_path);
    let dump_capacity = trace_policy.dump_capacity;
    install_policy(trace_policy);

    forsl_trace::init_subscriber(writer, dump_capacity, default_targets())
}

pub fn init_test_subscriber<W>(
    writer: W,
    test_storage: Arc<Mutex<Vec<String>>>,
) -> anyhow::Result<()>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    install_defaults();

    forsl_trace::init_test_subscriber(writer, 64, default_targets(), test_storage)
}

fn default_targets() -> Targets {
    Targets::new()
        .with_default(Level::DEBUG)
        .with_target("ogurpchik", Level::WARN)
}
