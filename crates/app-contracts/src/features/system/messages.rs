use std::rc::Rc;

use serde::Deserialize;

use super::model::SystemTool;

#[derive(Clone)]
pub enum SystemMsg {
    Missing(Rc<[SystemTool]>),
    Favourites {
        pinned: Rc<[SystemTool]>,
        frequent: Rc<[SystemTool]>,
    },
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct OpenTool(pub SystemTool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct GetTool(pub SystemTool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct PinTool(pub SystemTool, pub bool);
