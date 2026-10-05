use std::rc::Rc;

use guinea::prelude::*;

use super::messages::ActivityMsg;
use super::model::{ActivityRow, ActivityView, Filter, Group, Span};

#[derive(Clone, PartialEq, Debug)]
pub struct ActivityState {
    pub view: Load<Rc<ActivityView>>,
    pub span: Span,
    pub filter: Filter,
    pub hovered: Option<ActivityRow>,
    pub groups: Vec<Group>,
}

impl Default for ActivityState {
    fn default() -> Self {
        Self {
            view: Load::Loading,
            span: Span::default(),
            filter: Filter::default(),
            hovered: None,
            groups: Vec::new(),
        }
    }
}

#[reducer]
fn activity(this: &mut ActivityState, msg: ActivityMsg) {
    match msg {
        ActivityMsg::View(view) => this.view = Load::Ready(view),
        ActivityMsg::Span(span) => this.span = span,
        ActivityMsg::Filter(filter) => this.filter = filter,
        ActivityMsg::Hovered(row) => this.hovered = row,
        ActivityMsg::Groups(groups) => this.groups = groups,
    }
}
