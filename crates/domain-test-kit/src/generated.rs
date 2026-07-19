#![allow(unused)]

use forsl::native_windows::slint_factory::{RegistersSlintWindow, SlintWindowRegistry};

include!(concat!(env!("OUT_DIR"), "/generated_stubs.rs"));

impl RegistersSlintWindow for ServicesUiStub {
    fn register(&self, registry: &SlintWindowRegistry) {}
}
