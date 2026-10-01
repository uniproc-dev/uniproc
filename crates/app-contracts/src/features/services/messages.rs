use std::rc::Rc;

use serde::Deserialize;

use super::model::{ServiceActionKind, ServiceColumn, ServiceRow};
use crate::features::agents::ActionFailure;

#[derive(Clone)]
pub enum ServicesMsg {
    SetRows { rows: Rc<[ServiceRow]> },
    SetSelected(Option<String>),
    SetSort { column: ServiceColumn, descending: bool },
    SetFailure(Option<ActionFailure>),
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct DismissFailure;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Sort(pub ServiceColumn);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Select(pub String);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Deselect;

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct Command(pub ServiceActionKind);
