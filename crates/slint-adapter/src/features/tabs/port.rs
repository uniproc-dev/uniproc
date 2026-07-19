use crate::features::tabs::UiTabsAdapter;
use app_contracts::features::tabs::{UiTabsPort, UiTabsPortMsg};
use context::icons::resolve_image;
use context::page_status::PageStatus;
use forsl_macros::port_adapter;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

#[port_adapter(backend = "slint", window = AppWindow)]
impl UiTabsPort for UiTabsAdapter {
    fn send(&self, ui: &AppWindow, msg: UiTabsPortMsg) {
        match msg {
            UiTabsPortMsg::SetTabs(tabs) => {
                let slint_tabs: Vec<_> = tabs
                    .into_iter()
                    .map(|tab| {
                        let pages: Vec<_> = tab
                            .pages
                            .into_iter()
                            .map(|p| crate::PageData {
                                path: p.path.into(),
                                route_segment: p.route_segment.into(),
                                text: p.text.into(),
                                icon: resolve_image(p.icon_key.as_str()),
                                status: p.status.into(),
                                error_msg: p.error_msg.into(),
                            })
                            .collect();

                        crate::TabData {
                            id: 0,
                            context_key: tab.context_key.0.to_string().into(),
                            title: tab.title.into(),
                            icon: resolve_image(tab.icon_key.as_str()),
                            pages: ModelRc::new(VecModel::from(pages)),
                            status: tab.status.into(),
                            error_msg: tab.error_msg.into(),
                            is_closable: tab.is_closable,
                        }
                    })
                    .collect();
                let slint_tabs: Vec<_> = slint_tabs
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut tab)| {
                        tab.id = i as i32;
                        tab
                    })
                    .collect();

                ui.global::<crate::Tabs>()
                    .set_tabs(ModelRc::new(VecModel::from(slint_tabs)));
            }
            UiTabsPortMsg::SetAvailableContexts(contexts) => {
                let slint_contexts: Vec<_> = contexts
                    .into_iter()
                    .map(|context| crate::AvailableContextData {
                        context_key: context.context_key.0.to_string().into(),
                        title: context.title.into(),
                        icon: resolve_image(context.icon_key.as_str()),
                        status: context.status.into(),
                    })
                    .collect();

                ui.global::<crate::Tabs>()
                    .set_available_contexts(ModelRc::new(VecModel::from(slint_contexts)));
            }
            UiTabsPortMsg::SetActiveContext(context_key) => {
                let tabs = ui.global::<crate::Tabs>();
                if let Some(idx) = tabs
                    .get_tabs()
                    .iter()
                    .position(|t| t.context_key.as_str() == context_key.0.as_ref())
                {
                    tabs.set_active_tab_index(idx as i32);
                }
            }
            UiTabsPortMsg::SetActivePage { context_key, route_segment } => {
                let tabs = ui.global::<crate::Tabs>();
                if let Some(tab) = tabs
                    .get_tabs()
                    .iter()
                    .find(|t| t.context_key.as_str() == context_key.0.as_ref())
                {
                    if let Some(idx) = tab
                        .pages
                        .iter()
                        .position(|p| p.route_segment.as_str() == route_segment.as_str())
                    {
                        tabs.set_active_page_index(idx as i32);
                    }
                }
            }
            UiTabsPortMsg::SetRouteStatus { context_key, route_segment, status } => {
                if let Some(tab) = ui
                    .global::<crate::Tabs>()
                    .get_tabs()
                    .iter()
                    .find(|t| t.context_key.as_str() == context_key.0.as_ref())
                {
                    if let Some(idx) = tab
                        .pages
                        .iter()
                        .position(|p| p.route_segment.as_str() == route_segment.as_str())
                    {
                        if let Some(mut row) = tab.pages.row_data(idx) {
                            row.status = status.into();
                            tab.pages.set_row_data(idx, row);
                        }
                    }
                }
            }
            UiTabsPortMsg::SetRouteError { context_key, route_segment, msg } => {
                if let Some(tab) = ui
                    .global::<crate::Tabs>()
                    .get_tabs()
                    .iter()
                    .find(|t| t.context_key.as_str() == context_key.0.as_ref())
                {
                    if let Some(idx) = tab
                        .pages
                        .iter()
                        .position(|p| p.route_segment.as_str() == route_segment.as_str())
                    {
                        if let Some(mut row) = tab.pages.row_data(idx) {
                            row.error_msg = msg.into();
                            tab.pages.set_row_data(idx, row);
                        }
                    }
                }
            }
        }
    }
}

impl From<PageStatus> for crate::PageStatus {
    fn from(status: PageStatus) -> Self {
        match status {
            PageStatus::Loading => crate::PageStatus::Loading,
            PageStatus::Ready => crate::PageStatus::Ready,
            PageStatus::Error => crate::PageStatus::Error,
            PageStatus::Inactive => crate::PageStatus::Inactive,
        }
    }
}
