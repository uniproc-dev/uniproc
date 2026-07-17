#![cfg_attr(coverage, feature(coverage_attribute))]
#![cfg_attr(coverage, coverage(off))]

use build_utils::collector::with_recompile_trigger;
use proc_macro::TokenStream;
use syn::{ItemFn, ItemImpl, ItemTrait, parse_macro_input};

mod actor_manifest;
mod binder_gen;
mod features;
mod handler;
mod schema;
mod slint_macros;

#[proc_macro_attribute]
pub fn actor_manifest(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);
    actor_manifest::actor_manifest_impl(attr.into(), impl_block).into()
}

#[proc_macro_attribute]
pub fn capability(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
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

#[proc_macro_attribute]
pub fn slint_port(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_item = parse_macro_input!(item as ItemTrait);
    slint_macros::strip_trait_helper_attrs(trait_item)
}

#[proc_macro_attribute]
pub fn slint_bindings(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut trait_item = syn::parse_macro_input!(item as syn::ItemTrait);

    let binder_code = binder_gen::generate_binder(&trait_item);

    for item in &mut trait_item.items {
        if let syn::TraitItem::Fn(method) = item {
            slint_macros::strip_helper_attrs(&mut method.attrs);
        }
    }

    let output = quote::quote! {
        #trait_item
        #binder_code
    };

    output.into()
}

#[proc_macro_attribute]
pub fn slint_dto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    slint_macros::slint_dto_impl(item)
}

#[proc_macro_attribute]
pub fn slint_port_adapter(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);
    with_recompile_trigger(slint_macros::slint_port_adapter_impl(attr, impl_block).into()).into()
}

#[proc_macro_attribute]
pub fn slint_bindings_adapter(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);
    with_recompile_trigger(slint_macros::slint_bindings_adapter_impl(attr, impl_block).into())
        .into()
}
