use domain::features::activity::ActivityDeps;
use domain::features::agent_link::AgentLinkDeps;
use domain::features::processes::ProcessesDeps;
use domain::features::system::SystemDeps;
use domain::features::wsl::WslDeps;
use guinea::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct PlatformDeps {
    pub processes: ProcessesDeps,
    pub system: SystemDeps,
    pub wsl: WslDeps,
    pub agent_link: AgentLinkDeps,
    pub activity: ActivityDeps,
}

plugin! {
    pub Platform {
        provides { ProcessesDeps, SystemDeps, WslDeps, AgentLinkDeps, ActivityDeps }
    }
}

#[builds]
fn platform(_app: &mut PluginBuilder, deps: PlatformDeps) -> anyhow::Result<Platform> {
    Ok(Platform(deps.processes, deps.system, deps.wsl, deps.agent_link, deps.activity))
}
