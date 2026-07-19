use forsl_core::page_status::PageStatus;
use forsl_macros::port;

use super::TabContextKey;
use super::model::{AvailableContextDescriptor, TabDescriptor};

#[derive(Clone, Debug)]
pub enum UiTabsPortMsg {
    SetTabs(Vec<TabDescriptor>),
    SetAvailableContexts(Vec<AvailableContextDescriptor>),
    SetActiveContext(TabContextKey),
    SetActivePage { context_key: TabContextKey, route_segment: String },
    SetRouteStatus { context_key: TabContextKey, route_segment: String, status: PageStatus },
    SetRouteError { context_key: TabContextKey, route_segment: String, msg: String },
}

#[port]
pub trait UiTabsPort: 'static {
    fn send(&self, msg: UiTabsPortMsg);
}
