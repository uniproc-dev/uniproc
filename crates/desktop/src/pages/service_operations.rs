use std::time::Duration;

use ui::pages::services::{OperationsProps, operations};
use windows_reactor::{Component, ComponentContext, ComponentTimer, Grid, View, ViewContext};

pub(super) struct ServiceOperations {
    shown: OperationsProps,
    mounted: bool,
    visible: bool,
    collapsed: bool,
    timer: Option<ComponentTimer>,
}

pub(super) enum Message {
    Reveal,
    Remove,
    Toggle,
}

impl Component for ServiceOperations {
    type Input = OperationsProps;
    type Message = Message;

    fn create(input: &OperationsProps, cx: &ComponentContext<Self>) -> Self {
        let mounted = !input.operations.is_empty();
        Self {
            shown: input.clone(),
            mounted,
            visible: false,
            collapsed: false,
            timer: mounted.then(|| cx.set_timeout(Duration::from_millis(32), Message::Reveal)),
        }
    }

    fn input_changed(&mut self, input: &OperationsProps, cx: &ComponentContext<Self>) {
        if input.operations.is_empty() {
            self.shown.palette = input.palette;
            if self.visible {
                self.visible = false;
                self.timer = Some(cx.set_timeout(Duration::from_millis(220), Message::Remove));
            } else if self.mounted && !self.shown.operations.is_empty() && self.timer.is_some() {
                self.timer = Some(cx.set_timeout(Duration::from_millis(220), Message::Remove));
            }
        } else {
            self.shown = input.clone();
            if !self.visible {
                if !self.mounted {
                    self.collapsed = false;
                }
                self.mounted = true;
                self.timer = Some(cx.set_timeout(Duration::from_millis(32), Message::Reveal));
            }
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Reveal => {
                self.visible = true;
                self.timer = None;
            }
            Message::Remove => {
                self.mounted = false;
                self.timer = None;
            }
            Message::Toggle => self.collapsed = !self.collapsed,
        }
    }

    fn view(&self, _: &OperationsProps, cx: &mut ViewContext<Self>) -> View {
        if !self.mounted {
            return Grid::new().into();
        }
        operations(
            &self.shown,
            self.visible,
            self.collapsed,
            cx.callback(|_| Message::Toggle),
        )
    }
}
