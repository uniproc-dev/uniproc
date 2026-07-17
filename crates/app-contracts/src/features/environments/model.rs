use forsl_core::actor::Message;
use std::borrow::Cow;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WslDistroDto {
    pub name: String,
    pub is_installed: bool,
    pub is_running: bool,
    pub latency_ms: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentKind {
    Host,
    Wsl,
    Docker,
    Remote,
    Custom(Cow<'static, str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentStatus {
    Starting,
    Ready,
    Degraded,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentDescriptor {
    pub id: Cow<'static, str>,
    pub title: Cow<'static, str>,
    pub kind: EnvironmentKind,
    pub capabilities: Vec<Cow<'static, str>>,
    pub status: EnvironmentStatus,
}

#[derive(Debug, Clone)]
pub struct DiscoveryReport {
    pub provider_id: Cow<'static, str>,
    pub items: Vec<EnvironmentDescriptor>,
}
impl Message for DiscoveryReport {}

#[derive(Debug, Clone, Default)]
pub struct EnvironmentRegistryChanged {
    pub environments: Vec<EnvironmentDescriptor>,
}
impl Message for EnvironmentRegistryChanged {}
