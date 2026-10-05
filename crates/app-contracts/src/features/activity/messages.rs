use std::sync::Arc;

use serde::Deserialize;

use super::model::{ActivityRow, ActivityView, Area, Filter, Group, Hue, Pick, Span};
use crate::features::agents::ProcessInstance;

#[derive(Clone, PartialEq, Debug)]
pub enum ActivityMsg {
    View(Arc<ActivityView>),
    Span(Span),
    Filter(Filter),
    Hovered(Option<ActivityRow>),
    Groups(Vec<Group>),
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowGroup {
    pub group: String,
    pub shown: bool,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowOther(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct PutInGroup {
    pub group: String,
    pub rule: Pick,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct NewGroup(pub Pick);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct AddGroup(pub String);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct DropRule {
    pub group: String,
    pub rule: Pick,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RenameGroup {
    pub group: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct RecolorGroup {
    pub group: String,
    pub hue: Hue,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct MoveGroup {
    pub group: String,
    pub up: bool,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct DeleteGroup(pub String);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowSpan(pub Span);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowCame(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowWent(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct NewOnly(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ShowSeries(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Only(pub Option<Pick>);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Hide(pub Pick);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Unhide(pub Pick);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Search(pub String);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct PickArea(pub Area);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ClearArea;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Hover(pub Option<ProcessInstance>);
