use super::model::ResizeEdge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiWindowActionsPortMsg {
    Drag,
    Close,
    Minimize,
    ToggleMaximize,
    Resize(ResizeEdge),
}

pub trait UiWindowActionsPort: Clone + 'static {
    fn send(&self, msg: UiWindowActionsPortMsg);
}
