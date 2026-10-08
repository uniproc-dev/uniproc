#[allow(clippy::all, non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]
mod bindings {
    use crate::iterable::First;
    include!("bindings.rs");
}
mod body;
mod cache;
mod icons;
mod interop;
mod iterable;
mod paint;
mod scene;
mod text;

pub mod bar;
pub mod layout;
pub mod model;
pub mod nav;
pub mod realize;
pub mod trim;

pub use body::{body, Body};
pub use paint::Look;
