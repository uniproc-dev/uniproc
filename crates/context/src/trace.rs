pub use forsl_core::trace::{
    DispatchMeta, MetaGuard, TestCaptureLayer, TraceDumpLayer, current_correlation_id,
    current_meta, current_or_new_correlation_uuid, current_policy, format_ui_target_1,
    format_ui_target_2, in_named_scope, in_ui_action_scope, install_current_meta, install_policy,
    is_message_enabled, is_scope_enabled, is_target_enabled, normalize_policy, register_scopes,
};

use forsl_core::trace::{
    GeneratedScopes, TracePolicy, builtin_policy_from, init_traced_subscriber,
    init_traced_test_subscriber,
};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::writer::MakeWriter;

include!(concat!(env!("OUT_DIR"), "/trace_scopes.rs"));

const SCOPES: GeneratedScopes = GeneratedScopes {
    all: ALL_SCOPES,
    builtin_enable: BUILTIN_ENABLE_SCOPES,
    builtin_disable_messages: BUILTIN_DISABLE_MESSAGES,
    builtin_disable_targets: BUILTIN_DISABLE_TARGETS,
};

pub fn builtin_policy() -> TracePolicy {
    builtin_policy_from(&SCOPES)
}

pub fn init_subscriber<W>(settings_path: &Path, writer: W) -> anyhow::Result<()>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    init_traced_subscriber(settings_path, writer, &SCOPES, default_targets())
}

pub fn init_test_subscriber<W>(
    writer: W,
    test_storage: Arc<Mutex<Vec<String>>>,
) -> anyhow::Result<()>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    init_traced_test_subscriber(writer, test_storage, &SCOPES, default_targets())
}

fn default_targets() -> Targets {
    Targets::new()
        .with_default(Level::DEBUG)
        .with_target("ogurpchik", Level::WARN)
}
