use serde::Deserialize;

#[derive(Clone, Copy)]
pub enum SidebarMsg {
    Set { open: bool, width: u64 },
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Toggle;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SetOpen(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SetWidth(pub u64);
