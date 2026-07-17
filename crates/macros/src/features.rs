use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, parse_macro_input};

pub fn window_feature_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let func_name = &sig.ident;

    let struct_name_str = func_name.to_string().to_upper_camel_case();
    let struct_name = syn::Ident::new(&struct_name_str, func_name.span());

    let mut params_info = None;
    let mut ports = Vec::new();
    let mut args_iter = sig.inputs.iter();
    let _ctx_arg = args_iter.next();

    let mut port_idx = 1;
    for arg in args_iter {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                if pat_ident.ident == "params" {
                    params_info = Some((&pat_ident.ident, &pat_type.ty));
                } else {
                    ports.push((&pat_ident.ident, &pat_type.ty, port_idx));
                    port_idx += 1;
                }
            }
        }
    }

    let f_generics: Vec<_> = ports
        .iter()
        .map(|(_, _, idx)| syn::Ident::new(&format!("F{}", idx), proc_macro2::Span::call_site()))
        .collect();

    let struct_fields = ports.iter().map(|(_, _, idx)| {
        let field_name = syn::Ident::new(&format!("make_p{}", idx), proc_macro2::Span::call_site());
        let generic_f = syn::Ident::new(&format!("F{}", idx), proc_macro2::Span::call_site());
        quote! { #field_name: #generic_f }
    });

    let new_params = ports.iter().map(|(_, _, idx)| {
        let field_name = syn::Ident::new(&format!("make_p{}", idx), proc_macro2::Span::call_site());
        let generic_f = syn::Ident::new(&format!("F{}", idx), proc_macro2::Span::call_site());
        quote! { #field_name: #generic_f }
    });

    let new_initializers = ports.iter().map(|(_, _, idx)| {
        let field_name = syn::Ident::new(&format!("make_p{}", idx), proc_macro2::Span::call_site());
        quote! { #field_name }
    });

    let install_let_bindings = ports.iter().map(|(pat_ident, _, idx)| {
        let field_name = syn::Ident::new(&format!("make_p{}", idx), proc_macro2::Span::call_site());
        quote! { let #pat_ident = (self.#field_name)(ctx.ui); }
    });

    let call_args = sig.inputs.iter().skip(1).map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                if pat_ident.ident == "params" {
                    quote! { self.params.clone() }
                } else {
                    quote! { #pat_ident }
                }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        }
    });

    let original_generics = &sig.generics.params;
    let mut impl_where_predicates = Vec::new();

    if let Some(where_clause) = &sig.generics.where_clause {
        for pred in &where_clause.predicates {
            impl_where_predicates.push(quote! { #pred });
        }
    }

    for (i, (_, ty, idx)) in ports.iter().enumerate() {
        let generic_f = &f_generics[i];
        impl_where_predicates.push(quote! {
            #generic_f: Fn(&TWindow) -> #ty + Clone + 'static
        });
    }

    let struct_params_field = params_info.map(|(_, ty)| quote! { pub params: #ty, });
    let new_params_arg = params_info.map(|(_, ty)| quote! { params: #ty, });
    let new_params_init = params_info.map(|_| quote! { params, });

    let generics_bracket = if f_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#f_generics),*> }
    };

    let impl_generics = if f_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#f_generics),*> }
    };

    let expanded = quote! {
        #input_fn

        #vis struct #struct_name #generics_bracket {
            #struct_params_field
            #(#struct_fields),*
        }

        impl #impl_generics #struct_name #generics_bracket {
            pub fn new(#new_params_arg #(#new_params),*) -> Self {
                Self {
                    #new_params_init
                    #(#new_initializers),*
                }
            }
        }

        impl<#original_generics, #(#f_generics),*> WindowFeature<TWindow> for #struct_name #generics_bracket
        where
            #(#impl_where_predicates),*
        {
            fn install(&mut self, ctx: &mut WindowFeatureInitContext<TWindow>) -> anyhow::Result<()> {
                #(#install_let_bindings)*
                #func_name(ctx, #(#call_args),*)
            }
        }
    };

    expanded.into()
}

pub fn app_feature_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let func_name = &sig.ident;

    let struct_name_str = func_name.to_string().to_upper_camel_case();
    let struct_name = syn::Ident::new(&struct_name_str, func_name.span());

    let mut params_info = None;
    let mut args_iter = sig.inputs.iter();
    let _ctx_arg = args_iter.next();

    if let Some(FnArg::Typed(pat_type)) = args_iter.next() {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            if pat_ident.ident == "params" {
                params_info = Some((&pat_ident.ident, &pat_type.ty));
            } else {
                panic!("app_feature must only have 'ctx' and optionally 'params' as arguments");
            }
        }
    }
    if args_iter.next().is_some() {
        panic!("app_feature can have at most two arguments: 'ctx' and 'params'");
    }

    let expanded = if let Some((_, params_ty)) = params_info {
        quote! {
            #input_fn

            #vis struct #struct_name {
                pub params: #params_ty,
            }

            impl #struct_name {
                pub fn new(params: #params_ty) -> Self {
                    Self { params }
                }
            }

            impl AppFeature for #struct_name {
                fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
                    #func_name(ctx, self.params.clone())
                }
            }
        }
    } else {
        quote! {
            #input_fn

            #vis struct #struct_name;

            impl AppFeature for #struct_name {
                fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
                    #func_name(ctx)
                }
            }
        }
    };

    expanded.into()
}
