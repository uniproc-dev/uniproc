use std::rc::Rc;
use std::sync::Arc;

use crate::features::agents::{ProcessInstance, ScheduledTask};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Clock {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Default, Hash, serde::Deserialize)]
pub enum Span {
    Quarter,
    #[default]
    Hour,
    Connected,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Filter {
    pub came: bool,
    pub went: bool,
    pub new_only: bool,
    pub bursts: bool,
    pub text: String,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            came: true,
            went: true,
            new_only: false,
            bursts: true,
            text: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, serde::Deserialize)]
pub struct Area {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Dot {
    pub key: ProcessInstance,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Level {
    pub y: f32,
    pub lived: u64,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Scatter {
    pub dots: Vec<Dot>,
    pub orphans: Vec<Dot>,
    pub levels: Vec<Level>,
    pub alive_y: f32,
    pub ticks: Vec<Clock>,
    pub area: Option<Area>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Launcher {
    Task(ScheduledTask),
    Services(Arc<[Arc<str>]>),
    Process(Arc<str>),
    Unknown,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Exit {
    pub at: Clock,
    pub code: u32,
    pub lived: u64,
    pub cpu_cycles: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub peak_commit_bytes: u64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Came {
    pub key: ProcessInstance,
    pub at: Clock,
    pub name: Arc<str>,
    pub image_path: Arc<str>,
    pub command_line: Arc<str>,
    pub working_dir: Arc<str>,
    pub user: Arc<str>,
    pub session_id: u32,
    pub elevated: Option<bool>,
    pub launcher: Launcher,
    pub chain: Vec<Arc<str>>,
    pub parent_services: Arc<[Arc<str>]>,
    pub first_seen: bool,
    pub exit: Option<Exit>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Went {
    pub key: ProcessInstance,
    pub name: Option<Arc<str>>,
    pub lived: Option<u64>,
    pub exit: Exit,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Burst {
    pub key: ProcessInstance,
    pub at: Clock,
    pub launcher: Arc<str>,
    pub names: Vec<(Arc<str>, usize)>,
    pub went: usize,
    pub lasted: u64,
    pub members: Vec<Rc<Came>>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActivityRow {
    Came(Rc<Came>),
    Went(Rc<Went>),
    Burst(Rc<Burst>),
}

impl ActivityRow {
    pub fn key(&self) -> ProcessInstance {
        match self {
            Self::Came(came) => came.key,
            Self::Went(went) => went.key,
            Self::Burst(burst) => burst.key,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ActivityView {
    pub scatter: Rc<Scatter>,
    pub rows: Vec<ActivityRow>,
    pub earlier: usize,
    pub came: usize,
    pub went: usize,
    pub from: Clock,
    pub to: Clock,
    pub history_since: Option<Clock>,
    pub lost: u64,
}
