use std::rc::Rc;

use serde::Deserialize;

use super::model::{ActivityRow, ActivityView, Area, Filter, Pick, Span};
use crate::features::agents::ProcessInstance;

#[derive(Clone, PartialEq, Debug)]
pub enum ActivityMsg {
    View(Rc<ActivityView>),
    Span(Span),
    Filter(Filter),
    Hovered(Option<ActivityRow>),
}

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
