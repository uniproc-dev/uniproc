use build_utils::load_schema;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Fields, FieldsNamed, FieldsUnnamed, Ident, ImplItem, ItemImpl, ItemStruct, Meta,
    Path, Token, Type, Visibility, parse_quote,
};

enum ManifestItem {
    New(ItemStruct),
    Existing(Type),
    Group(Punctuated<ParsedItem, Token![,]>),
}

struct ParsedItem {
    attrs: Vec<Attribute>,
    kind: ManifestItem,
}

impl ParsedItem {
    fn parse_with_context(input: ParseStream, force_new: bool) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        if input.peek(Ident) && input.peek2(syn::token::Brace) {
            let fork = input.fork();
            let name: Ident = fork.parse()?;
            if name == "bind" {
                let _: Ident = input.parse()?;
                let content;
                syn::braced!(content in input);

                let mut inner = Punctuated::new();
                while !content.is_empty() {
                    let item = Self::parse_with_context(&content, force_new)?;
                    inner.push_value(item);
                    if content.is_empty() {
                        break;
                    }
                    inner.push_punct(content.parse::<Token![,]>()?);
                }

                return Ok(ParsedItem {
                    attrs,
                    kind: ManifestItem::Group(inner),
                });
            }
        }

        if input.peek(Token![@]) {
            let _: Token![@] = input.parse()?;
            let ty: Type = input.parse()?;
            return Ok(ParsedItem {
                attrs,
                kind: ManifestItem::Existing(ty),
            });
        }

        if input.peek(Token![struct]) || (input.peek(Token![pub]) && input.peek2(Token![struct])) {
            let mut s: ItemStruct = input.parse()?;
            s.attrs = Vec::new();
            if let Fields::Unit = s.fields {
                s.semi_token = Some(Default::default());
            }
            return Ok(ParsedItem {
                attrs,
                kind: ManifestItem::New(s),
            });
        }

        let _: Visibility = input.parse().unwrap_or(Visibility::Inherited);

        if input.peek(Ident) && (input.peek2(syn::token::Paren) || input.peek2(syn::token::Brace)) {
            let ident: Ident = input.parse()?;
            if input.peek(syn::token::Paren) {
                let mut fields: FieldsUnnamed = input.parse()?;
                for f in &mut fields.unnamed {
                    f.vis = syn::parse_quote!(pub);
                }
                Ok(ParsedItem {
                    attrs,
                    kind: ManifestItem::New(create_struct(ident, Fields::Unnamed(fields))),
                })
            } else {
                let mut fields: FieldsNamed = input.parse()?;
                for f in &mut fields.named {
                    f.vis = syn::parse_quote!(pub);
                }
                Ok(ParsedItem {
                    attrs,
                    kind: ManifestItem::New(create_struct(ident, Fields::Named(fields))),
                })
            }
        } else {
            let ty: Type = input.parse()?;
            let maybe_ident = if let Type::Path(ref p) = ty {
                if p.qself.is_none() && p.path.segments.len() == 1 {
                    let seg = &p.path.segments[0];
                    if let syn::PathArguments::None = seg.arguments {
                        Some(seg.ident.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if force_new {
                if let Some(ident) = maybe_ident {
                    Ok(ParsedItem {
                        attrs,
                        kind: ManifestItem::New(create_struct(ident, Fields::Unit)),
                    })
                } else {
                    Ok(ParsedItem {
                        attrs,
                        kind: ManifestItem::Existing(ty),
                    })
                }
            } else {
                Ok(ParsedItem {
                    attrs,
                    kind: ManifestItem::Existing(ty),
                })
            }
        }
    }
}

fn create_struct(ident: Ident, fields: Fields) -> ItemStruct {
    let semi = if let Fields::Named(_) = fields {
        None
    } else {
        Some(Default::default())
    };
    ItemStruct {
        attrs: Vec::new(),
        vis: syn::parse_quote!(pub),
        struct_token: Default::default(),
        ident,
        generics: Default::default(),
        fields,
        semi_token: semi,
    }
}

struct TransformResult {
    marker_type: Type,
    generated_structs: Vec<TokenStream>,
    logic_calls: Vec<TokenStream>,
}

fn process_manifest_items(
    items: Punctuated<ParsedItem, Token![,]>,
    force_bind: bool,
    is_bus: bool,
    self_ty: &Type,
    bound_messages: &mut Vec<Ident>,
    generated_structs: &mut Vec<TokenStream>,
    logic_calls: &mut Vec<TokenStream>,
) {
    for it in items {
        let mut attrs = it.attrs;

        let mut bind_attr = None;
        attrs.retain(|a| {
            if a.path().is_ident("bind") {
                bind_attr = Some(a.clone());
                false
            } else {
                true
            }
        });

        let is_bind = bind_attr.is_some() || force_bind;
        let is_manual = bind_attr
            .map(|a| {
                if let Meta::List(list) = &a.meta {
                    list.tokens.to_string() == "manual"
                } else {
                    false
                }
            })
            .unwrap_or(false);

        match it.kind {
            ManifestItem::Group(inner_items) => {
                process_manifest_items(
                    inner_items,
                    true,
                    is_bus,
                    self_ty,
                    bound_messages,
                    generated_structs,
                    logic_calls,
                );
            }
            ManifestItem::New(s) => {
                let id = &s.ident;
                if is_bind {
                    bound_messages.push(id.clone());
                }

                generated_structs.push(quote! {
                    #(#attrs)* #[derive(Debug, Clone)] #s
                    #(#attrs)* impl app_core::actor::Message for #id {}
                });

                if is_bind && !is_manual {
                    let arity = s.fields.len();
                    let types: Vec<Type> = s.fields.iter().map(|f| f.ty.clone()).collect();

                    let args_tuple = if arity == 0 {
                        quote!(())
                    } else if arity == 1 {
                        let ty = &types[0];
                        quote!(#ty)
                    } else {
                        quote!((#(#types),*))
                    };

                    let destr = if arity == 0 {
                        quote!(_)
                    } else if arity == 1 {
                        quote!(arg0)
                    } else {
                        let arg_ids: Vec<_> =
                            (0..arity).map(|i| format_ident!("arg{}", i)).collect();
                        quote!((#(#arg_ids),*))
                    };

                    let body = match &s.fields {
                        Fields::Named(f) => {
                            let field_assigns = f.named.iter().enumerate().map(|(i, field)| {
                                let id = &field.ident;
                                if arity == 1 {
                                    quote! { #id: arg0 }
                                } else {
                                    let arg = format_ident!("arg{}", i);
                                    quote! { #id: #arg }
                                }
                            });
                            quote! { Self { #(#field_assigns),* } }
                        }
                        Fields::Unnamed(_) => {
                            let field_assigns = (0..arity).map(|i| {
                                if arity == 1 {
                                    quote! { arg0 }
                                } else {
                                    format_ident!("arg{}", i).to_token_stream()
                                }
                            });
                            quote! { Self(#(#field_assigns),*) }
                        }
                        Fields::Unit => quote! { Self },
                    };

                    generated_structs.push(quote! {
                        impl std::convert::From<#args_tuple> for #id {
                            fn from(#destr: #args_tuple) -> Self { #body }
                        }
                    });
                }

                let item_ty = quote!(#id);
                if is_bus {
                    logic_calls.push(quote! {
                        #(#attrs)*
                        <#item_ty as app_core::actor::event_bus::builder::EventSubscription<#self_ty>>::subscribe_into(addr.clone(), tracker);
                    });
                } else {
                    logic_calls.push(quote! {
                        #(#attrs)*
                        assert_handler::<#self_ty, #item_ty>();
                    });
                }
            }
            ManifestItem::Existing(t) => {
                let item_ty = quote!(#t);
                if is_bus {
                    logic_calls.push(quote! {
                        #(#attrs)*
                        <#item_ty as app_core::actor::event_bus::builder::EventSubscription<#self_ty>>::subscribe_into(addr.clone(), tracker);
                    });
                } else {
                    logic_calls.push(quote! {
                        #(#attrs)*
                        assert_handler::<#self_ty, #item_ty>();
                    });
                }
            }
        }
    }
}

fn transform_manifest(
    ty: &mut Type,
    mac_name: &str,
    force_new: bool,
    self_ty: &Type,
    marker_ident: Ident,
    is_bus: bool,
    bound_messages: &mut Vec<Ident>,
) -> Option<TransformResult> {
    let mut generated_structs = Vec::new();
    let mut logic_calls = Vec::new();

    let tokens = match ty {
        Type::Macro(m) if m.mac.path.is_ident(mac_name) => m.mac.tokens.clone(),
        Type::Path(p) => {
            for seg in &mut p.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut seg.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            if let Some(res) = transform_manifest(
                                inner,
                                mac_name,
                                force_new,
                                self_ty,
                                marker_ident.clone(),
                                is_bus,
                                bound_messages,
                            ) {
                                return Some(res);
                            }
                        }
                    }
                }
            }
            return None;
        }
        _ => return None,
    };

    let items = (move |input: ParseStream| {
        let mut punctuated = Punctuated::new();
        while !input.is_empty() {
            let item = ParsedItem::parse_with_context(input, force_new)?;
            punctuated.push_value(item);
            if input.is_empty() {
                break;
            }
            let punct: Token![,] = input.parse()?;
            punctuated.push_punct(punct);
        }
        Ok(punctuated)
    })
    .parse2(tokens)
    .expect("Failed to parse manifest macro");

    process_manifest_items(
        items,
        false,
        is_bus,
        self_ty,
        bound_messages,
        &mut generated_structs,
        &mut logic_calls,
    );

    *ty = syn::parse_quote!(#marker_ident);

    Some(TransformResult {
        marker_type: syn::parse_quote!(#marker_ident),
        generated_structs,
        logic_calls,
    })
}

struct ManifestArgs {
    binder_path: Option<Path>,
    ui_bind_partial: bool,
}

fn parse_manifest_args(attr: TokenStream) -> ManifestArgs {
    let mut args = ManifestArgs {
        binder_path: None,
        ui_bind_partial: false,
    };
    let parser = syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated;
    if let Ok(metas) = parser.parse2(attr) {
        for meta in metas {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("binder") => {
                    if let syn::Expr::Path(p) = &nv.value {
                        args.binder_path = Some(p.path.clone());
                    }
                }
                Meta::Path(p) if p.is_ident("ui_bind_partial") => {
                    args.ui_bind_partial = true;
                }
                _ => {}
            }
        }
    }
    args
}

pub fn actor_manifest_impl(attr: TokenStream, mut impl_block: ItemImpl) -> TokenStream {
    let manifest_args = parse_manifest_args(attr);
    let self_ty = &impl_block.self_ty;
    let (impl_generics, type_generics, where_clause) = impl_block.generics.split_for_impl();

    let base_name = quote!(#self_ty)
        .to_string()
        .replace(" ", "")
        .replace("<", "_")
        .replace(">", "_")
        .replace("::", "_");

    let mut has_bus = false;
    let mut has_signals = false;

    let mut all_structs = Vec::new();
    let mut signals_logic = quote! {};
    let mut bus_logic = quote! {};
    let mut handlers_logic = quote! {};
    let mut bound_messages = Vec::new();

    for item in &mut impl_block.items {
        if let ImplItem::Type(ty_item) = item {
            if ty_item.ident == "Bus" {
                has_bus = true;
                let marker_id = format_ident!("__Bus_{}", base_name);
                if let Some(res) = transform_manifest(
                    &mut ty_item.ty,
                    "bus",
                    false,
                    self_ty,
                    marker_id.clone(),
                    true,
                    &mut Vec::new(),
                ) {
                    all_structs.extend(res.generated_structs);
                    let calls = res.logic_calls;
                    bus_logic = quote! {
                        #[doc(hidden)] pub struct #marker_id;
                        impl #impl_generics app_core::actor::event_bus::builder::EventSubscription<#self_ty> for #marker_id #where_clause {
                            fn subscribe_into(addr: app_core::actor::Addr<#self_ty>, tracker: &impl app_core::lifecycle_tracker::LifecycleTracker) {
                                #(#calls)*
                            }
                        }
                    };
                }
            }

            if ty_item.ident == "Signals" {
                has_signals = true;
                let marker_id = format_ident!("__Signals_{}", base_name);

                let msgs = extract_bus_types(&ty_item.ty);

                ty_item.ty = syn::parse_quote!(#marker_id);

                let sig_impls = msgs.iter().map(|msg_ty| {
                    quote! {
                        impl app_core::actor::traits::AllowedSignal<#msg_ty> for #marker_id {}
                    }
                });

                signals_logic = quote! {
                    #[doc(hidden)]
                    pub struct #marker_id;
                    #(#sig_impls)*
                };
            }

            if ty_item.ident == "Handlers" {
                let marker_id = format_ident!("__Handlers_{}", base_name);
                if let Some(res) = transform_manifest(
                    &mut ty_item.ty,
                    "handlers",
                    true,
                    self_ty,
                    marker_id.clone(),
                    false,
                    &mut bound_messages,
                ) {
                    all_structs.extend(res.generated_structs);
                    let checks = res.logic_calls;
                    handlers_logic = quote! {
                        #[doc(hidden)] pub struct #marker_id;
                        impl #impl_generics app_core::actor::DirectHandler<#self_ty> for #marker_id #where_clause {}
                        const _: () = {
                            fn check_handlers #impl_generics () #where_clause {
                                fn assert_handler<A, M>() where A: app_core::actor::Handler<M>, M: app_core::actor::Message {}
                                #(#checks)*
                            }
                        };
                    };
                }
            }
        }
    }

    if !has_bus {
        impl_block.items.push(parse_quote!(
            type Bus = ();
        ));
    }
    if !has_signals {
        impl_block.items.push(parse_quote!(
            type Signals = ();
        ));
    }

    let mut auto_bind_logic = quote! {};
    let mut summary_doc = String::new();
    if let Some(binder_path) = manifest_args.binder_path {
        let last_seg = binder_path.segments.last().unwrap().ident.to_string();
        let binder_base = last_seg.strip_suffix("Binder").unwrap_or(&last_seg);
        let expected_trait_name = format!("Ui{}Bindings", binder_base);

        let mut bindings_path = binder_path.clone();
        bindings_path.segments.last_mut().unwrap().ident = format_ident!("{}", expected_trait_name);

        let schema = load_schema();
        let trait_def = schema
            .bindings
            .iter()
            .find(|b| b.name == expected_trait_name)
            .unwrap_or_else(|| panic!("Trait {} not found in schema", expected_trait_name));

        use heck::{ToSnakeCase, ToUpperCamelCase};
        let available_methods: Vec<&str> =
            trait_def.methods.iter().map(|m| m.name.as_str()).collect();

        for msg_ident in &bound_messages {
            let msg_name = msg_ident.to_string();
            let expected_method_name = format!("on_{}", msg_name.to_snake_case());

            if !available_methods.contains(&expected_method_name.as_str()) {
                let suggestion = build_utils::suggest_closest(
                    &expected_method_name,
                    available_methods.iter().cloned(),
                );

                let err = if let Some(best_match) = suggestion {
                    let struct_suggestion = best_match
                        .strip_prefix("on_")
                        .unwrap_or(best_match)
                        .to_upper_camel_case();

                    format!(
                        "Message `{msg_name}` is marked with #[bind], but method `{expected_method_name}` is missing in `{expected_trait_name}`.\n\
                         help: did you mean struct `{struct_suggestion}` (to match method `{best_match}`)?",
                        msg_name = msg_name,
                        expected_method_name = expected_method_name,
                        expected_trait_name = expected_trait_name,
                        struct_suggestion = struct_suggestion,
                        best_match = best_match
                    )
                } else {
                    format!(
                        "Message `{msg_name}` is marked with #[bind], but method `{expected_method_name}` is missing in `{expected_trait_name}`.\n\
                         help: available methods: {}",
                        available_methods.join(", ")
                    )
                };
                return syn::Error::new_spanned(msg_ident, err)
                    .to_compile_error()
                    .into();
            }
        }

        let mut missing_methods = Vec::new();
        let mut bind_calls = Vec::new();
        let mut bind_summary = Vec::new();

        for method in &trait_def.methods {
            let expected_msg = method
                .name
                .strip_prefix("on_")
                .unwrap_or(&method.name)
                .to_upper_camel_case();

            if let Some(msg_id) = bound_messages
                .iter()
                .find(|m| m.to_string() == expected_msg)
            {
                let method_ident = format_ident!("{}", method.name);
                bind_calls.push(quote! { .#method_ident::<#msg_id>() });
                bind_summary.push(format!("* [x] **{}** → `{}`", msg_id, method.name));
            } else {
                missing_methods.push((method.name.clone(), expected_msg));
                bind_summary.push(format!("* [ ] (missing) → `{}`", method.name));
            }
        }

        if !missing_methods.is_empty() && !manifest_args.ui_bind_partial {
            let mut error_msg = format!(
                "Actor implementation is incomplete for `{}`.\n\
                 Missing bindings for the following methods:\n",
                expected_trait_name
            );

            for (method_name, struct_name) in missing_methods {
                error_msg.push_str(&format!(
                    "  • {}  =>  expected struct `{}` with #[bind]\n",
                    method_name, struct_name
                ));
            }

            error_msg.push_str("\nhelp: implement these messages or add `ui_bind_partial` to #[actor_manifest] attribute");

            return syn::Error::new_spanned(&binder_path, error_msg)
                .to_compile_error()
                .into();
        }

        summary_doc = format!(
            "### UI Bindings Summary (`{}`)\n\n{}\n",
            expected_trait_name,
            bind_summary.join("\n")
        );

        let summary_attrs: Vec<_> = summary_doc
            .lines()
            .map(|line| quote! { #[doc = #line] })
            .collect();

        let is_complete = missing_methods.is_empty();

        let mut partial_binder_path = binder_path.clone();
        let last = partial_binder_path.segments.last_mut().unwrap();
        last.ident = format_ident!(
            "{}",
            last.ident.to_string().replace("Binder", "PartialBinder")
        );

        let mut bindings_trait = binder_path.clone();
        let binder_name = binder_path.segments.last().unwrap().ident.to_string();

        let trait_name = if binder_name.ends_with("Binder") {
            format!("Ui{}Bindings", binder_name.strip_suffix("Binder").unwrap())
        } else {
            binder_name.clone()
        };

        bindings_trait.segments.last_mut().unwrap().ident = format_ident!("{}", trait_name);

        let mut extended_generics = impl_block.generics.clone();
        let port_type_param: syn::TypeParam = parse_quote!(B_PORT: #bindings_trait);
        extended_generics
            .params
            .push(syn::GenericParam::Type(port_type_param));

        let (impl_extended, _, _) = extended_generics.split_for_impl();

        let wire_full_call = if is_complete {
            quote! {
                #binder_path::new(addr, port)
                    #(#bind_calls)*
                    .build();
            }
        } else {
            quote! {}
        };

        auto_bind_logic = quote! {

            impl #impl_extended framework::addr::UiAutoWire<B_PORT> for #self_ty #where_clause {
                const IS_COMPLETE: bool = #is_complete;

                #(#summary_attrs)*
                fn wire_full(addr: &app_core::actor::Addr<Self>, port: &B_PORT) {
                    #wire_full_call
                }

                #(#summary_attrs)*
                fn wire_partial(addr: &app_core::actor::Addr<Self>, port: &B_PORT) {
                    #partial_binder_path::new(addr, port)
                        #(#bind_calls)*
                        .build();
                }
            }

        };
    }

    if !summary_doc.is_empty() {
        for item in &mut impl_block.items {
            if let ImplItem::Type(ty) = item {
                if ty.ident == "Handlers" {
                    ty.attrs.push(parse_quote!(#[doc = #summary_doc]));
                }
            }
        }
    }

    quote! {
        #(#all_structs)*
        #bus_logic
        #signals_logic
        #handlers_logic
        #auto_bind_logic
        #impl_block
    }
    .into()
}

fn extract_bus_types(ty: &syn::Type) -> Vec<syn::Type> {
    if let syn::Type::Macro(m) = ty {
        if m.mac.path.is_ident("bus") {
            let parser = syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated;
            if let Ok(punctuated) = m.mac.parse_body_with(parser) {
                return punctuated.into_iter().collect();
            }
        }
    }
    Vec::new()
}
