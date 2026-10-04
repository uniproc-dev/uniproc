use std::rc::Rc;
use std::sync::Arc;

use crate::features::agents::{ProcessInstance, ScheduledTask};
use crate::ids::ids;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Clock {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

ids! {
    #[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Default, Hash, serde::Deserialize)]
    pub enum Span {
        Quarter => "quarter",
        #[default]
        Hour => "hour",
        Connected => "connected",
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub enum Pick {
    Exe(Arc<str>),
    Folder(Arc<str>),
    Launcher(Arc<str>),
}

impl Pick {
    pub fn id(&self) -> String {
        match self {
            Self::Exe(path) => format!("exe:{path}"),
            Self::Folder(path) => format!("folder:{path}"),
            Self::Launcher(name) => format!("launcher:{name}"),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let (kind, value) = id.split_once(':')?;
        match kind {
            "exe" => Some(Self::Exe(value.into())),
            "folder" => Some(Self::Folder(value.into())),
            "launcher" => Some(Self::Launcher(value.into())),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Filter {
    pub came: bool,
    pub went: bool,
    pub new_only: bool,
    pub series: bool,
    pub text: String,
    pub only: Option<Pick>,
    pub hidden: Vec<Pick>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            came: true,
            went: true,
            new_only: false,
            series: true,
            text: String::new(),
            only: None,
            hidden: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, serde::Deserialize)]
pub enum Lived {
    #[default]
    Unknown,
    For(u64),
    Running,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Deserialize)]
pub struct Area {
    pub from: u64,
    pub to: u64,
    pub shortest: Lived,
    pub longest: Lived,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dot {
    pub key: ProcessInstance,
    pub at: u64,
    pub lived: Lived,
    pub faint: bool,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Scatter {
    pub dots: Vec<Dot>,
    pub now: u64,
    pub now_clock: Clock,
    pub length: u64,
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
    pub picks: Vec<Pick>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Went {
    pub key: ProcessInstance,
    pub name: Option<Arc<str>>,
    pub lived: Option<u64>,
    pub exit: Exit,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Series {
    pub key: ProcessInstance,
    pub at: Clock,
    pub launcher: Arc<str>,
    pub folder: Arc<str>,
    pub names: Vec<(Arc<str>, usize)>,
    pub count: usize,
    pub went: usize,
    pub routine: bool,
    pub members: Vec<Rc<Came>>,
    pub picks: Vec<Pick>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActivityRow {
    Came(Rc<Came>),
    Went(Rc<Went>),
    Series(Rc<Series>),
}

impl ActivityRow {
    pub fn key(&self) -> ProcessInstance {
        match self {
            Self::Came(came) => came.key,
            Self::Went(went) => went.key,
            Self::Series(series) => series.key,
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
