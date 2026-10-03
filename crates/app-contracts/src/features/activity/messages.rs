use std::rc::Rc;

use serde::Deserialize;

use super::model::{ActivityView, Filter, Span};

#[derive(Clone, PartialEq, Debug)]
pub enum ActivityMsg {
    View(Rc<ActivityView>),
    Span(Span),
    Filter(Filter),
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
pub struct ShowBursts(pub bool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Search(pub String);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct PickBucket(pub usize);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct ClearRange;
