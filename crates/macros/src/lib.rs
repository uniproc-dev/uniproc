#![cfg_attr(coverage, feature(coverage_attribute))]
#![cfg_attr(coverage, coverage(off))]

use proc_macro::TokenStream;
use syn::{ItemFn, ItemImpl, parse_macro_input};

mod actor_manifest;
mod features;
mod handler;

#[proc_macro_attribute]
pub fn actor_manifest(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);
    actor_manifest::actor_manifest_impl(attr.into(), impl_block).into()
}

#[proc_macro_attribute]
pub fn window_feature(args: TokenStream, input: TokenStream) -> TokenStream {
    features::window_feature_impl(args, input)
}
#[proc_macro_attribute]
pub fn app_feature(args: TokenStream, input: TokenStream) -> TokenStream {
    features::app_feature_impl(args, input)
}

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    handler::generate_standalone_handler(input)
}
