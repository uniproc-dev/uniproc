use app_contracts::features::cosmetics::{AccentPalette, UiCosmeticsPort, UiCosmeticsPortMsg};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct CosmeticsPortStub {
    messages: Rc<RefCell<Vec<UiCosmeticsPortMsg>>>,
}

impl CosmeticsPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiCosmeticsPortMsg> {
        self.messages.borrow().clone()
    }

    pub fn last_accent_palette(&self) -> Option<AccentPalette> {
        self.messages.borrow().iter().rev().find_map(|msg| match msg {
            UiCosmeticsPortMsg::SetAccentPalette(palette) => Some(*palette),
            _ => None,
        })
    }

    pub fn apply_main_window_effects_called(&self) -> bool {
        self.messages
            .borrow()
            .iter()
            .any(|msg| matches!(msg, UiCosmeticsPortMsg::ApplyMainWindowEffects))
    }
}

impl UiCosmeticsPort for CosmeticsPortStub {
    fn send(&self, msg: UiCosmeticsPortMsg) {
        self.messages.borrow_mut().push(msg);
    }
}
