use guinea::prelude::*;
use std::rc::Rc;

use super::messages::ServicesMsg;
use super::model::{ServiceColumn, ServiceRow};

#[derive(Clone, PartialEq, Debug)]
pub struct ServicesState {
    pub rows: Load<Rc<[ServiceRow]>>,
    pub selected: Option<String>,
    pub sort_column: ServiceColumn,
    pub descending: bool,
}

impl Default for ServicesState {
    fn default() -> Self {
        Self {
            rows: Load::Loading,
            selected: None,
            sort_column: ServiceColumn::Name,
            descending: false,
        }
    }
}

impl ServicesState {
    pub fn rows(&self) -> &[ServiceRow] {
        self.rows.ready().map(|r| r.as_ref()).unwrap_or(&[])
    }

    pub fn total(&self) -> usize {
        self.rows().len()
    }
}

#[reducer]
fn services(this: &mut ServicesState, update: ServicesMsg) {
    match update {
        ServicesMsg::SetRows { rows } => this.rows = Load::Ready(rows),
        ServicesMsg::SetSelected(name) => this.selected = name,
        ServicesMsg::SetSort { column, descending } => {
            this.sort_column = column;
            this.descending = descending;
        }
    }
}
