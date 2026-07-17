mod settings;

use self::settings::TraceSettings;
use forsl_core::signal::SignalSubscription;
use forsl_core::trace::TracePolicy;
use forsl::feature::{AppFeature, AppFeatureInitContext, ContextStoreExt};
use macros::app_feature;
use std::sync::Arc;

#[app_feature]
pub fn trace_settings_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let store = ctx.store();

    let settings = TraceSettings::new(&store)?;
    apply_trace_policy(&settings);

    let mut subs = Vec::new();

    {
        let settings = settings.clone();
        subs.push(
            settings
                .enable_scopes()
                .subscribe(move |_| apply_trace_policy(&settings)),
        );
    }
    {
        let settings = settings.clone();
        subs.push(
            settings
                .disable_scopes()
                .subscribe(move |_| apply_trace_policy(&settings)),
        );
    }
    {
        let settings = settings.clone();
        subs.push(
            settings
                .disable_messages()
                .subscribe(move |_| apply_trace_policy(&settings)),
        );
    }
    {
        let settings = settings.clone();
        subs.push(
            settings
                .disable_targets()
                .subscribe(move |_| apply_trace_policy(&settings)),
        );
    }
    {
        let settings = settings.clone();
        subs.push(
            settings
                .dump_capacity()
                .subscribe(move |_| apply_trace_policy(&settings)),
        );
    }

    ctx.shared
        .insert_arc(Arc::new(TraceSettingsRuntime { _subs: subs }));
    Ok(())
}

struct TraceSettingsRuntime {
    _subs: Vec<SignalSubscription>,
}

fn apply_trace_policy(settings: &TraceSettings) {
    let builtin = context::trace::builtin_policy();
    let policy = context::trace::normalize_policy(TracePolicy {
        enabled_prefixes: merge_trace_values(
            builtin.enabled_prefixes,
            settings.enable_scopes().get(),
        ),
        disabled_prefixes: settings.disable_scopes().get(),
        disabled_message_prefixes: merge_trace_values(
            builtin.disabled_message_prefixes,
            settings.disable_messages().get(),
        ),
        disabled_target_prefixes: merge_trace_values(
            builtin.disabled_target_prefixes,
            settings.disable_targets().get(),
        ),
        dump_capacity: settings.dump_capacity().get() as usize,
    });

    forsl_core::trace::install_policy(policy);
}

fn merge_trace_values(mut builtin: Vec<String>, user: Vec<String>) -> Vec<String> {
    let mut values = Vec::new();
    values.append(&mut builtin);
    values.extend(user);
    values.sort();
    values.dedup();
    values
}
