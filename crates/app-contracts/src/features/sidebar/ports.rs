#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiSidebarPortMsg {
    SetSwitchTransition { from_index: i32, to_index: i32, progress: f32 },
    SetSideBarWidth(u64),
    SetSwitchProgress(f32),
    SetContentVisible(bool),
}

pub trait UiSidebarPort: 'static {
    fn send(&self, msg: UiSidebarPortMsg);
}
