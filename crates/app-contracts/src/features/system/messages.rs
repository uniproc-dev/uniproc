use std::rc::Rc;

use serde::Deserialize;

use super::model::SystemTool;

#[derive(Clone)]
pub enum SystemMsg {
    Set { missing: Rc<[SystemTool]> },
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct OpenTool(pub SystemTool);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct GetTool(pub SystemTool);
