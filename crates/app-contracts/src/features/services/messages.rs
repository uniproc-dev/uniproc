use std::rc::Rc;

use serde::Deserialize;

use super::model::{ServiceActionKind, ServiceColumn, ServiceRow};

#[derive(Clone)]
pub enum ServicesMsg {
    SetRows { rows: Rc<[ServiceRow]> },
    SetSelected(Option<String>),
    SetSort { column: ServiceColumn, descending: bool },
}

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
