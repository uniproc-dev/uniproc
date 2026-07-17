use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Error, FnArg, GenericArgument, ItemFn, PatType, PathArguments, Result, Type, spanned::Spanned,
};

pub fn generate_standalone_handler(item: ItemFn) -> TokenStream {
    expand_handler(item).unwrap_or_else(|err| err.to_compile_error().into())
}

fn expand_handler(item: ItemFn) -> Result<TokenStream> {
    let fn_name = &item.sig.ident;
    let is_async = item.sig.asyncness.is_some();
    let inputs = &item.sig.inputs;

    let (actor_ty, actor_name) = if is_async {
        extract_actor_from_async_ctx(inputs.get(0))?
    } else {
        extract_actor_from_mut(inputs.get(0))?
    };

    let (msg_ty, msg_name) = extract_msg_info(inputs.get(1))?;

    let handler_body = if is_async {
        if inputs.len() != 2 {
            return Err(Error::new(
                item.sig.span(),
                format!(
                    "Async handler for actor '{}' and message '{}' must have exactly 2 arguments: (ctx: AsyncContext<{}>, msg: {})",
                    actor_name, msg_name, actor_name, msg_name
                ),
            ));
        }

        quote! {
            let actx = ctx.async_ctx();
            tokio::spawn(async move {
                #fn_name(actx, msg).await;
            });
        }
    } else {
        if inputs.len() < 2 || inputs.len() > 3 {
            return Err(Error::new(
                item.sig.span(),
                format!(
                    "Sync handler for actor '{}' and message '{}' must have 2 or 3 arguments: (actor: &mut {}, msg: {}, [ctx: &Context<{}>])",
                    actor_name, msg_name, actor_name, msg_name, actor_name
                ),
            ));
        }

        let has_ctx = inputs.len() == 3;
        let call_args = if has_ctx {
            quote! { self, msg, ctx }
        } else {
            quote! { self, msg }
        };

        quote! { #fn_name(#call_args); }
    };

    let trait_generics = item.sig.generics.clone();
    let (impl_generics, _, where_clause) = trait_generics.split_for_impl();

    Ok(TokenStream::from(quote! {
        #item

        impl #impl_generics forsl_core::actor::Handler<#msg_ty> for #actor_ty #where_clause {
            fn handle(&mut self, msg: #msg_ty, ctx: &forsl_core::actor::Context<Self>) {
                #handler_body
            }
        }
    }))
}

fn type_to_string(ty: &Type) -> String {
    quote!(#ty).to_string().replace(' ', "")
}

fn extract_msg_info(arg: Option<&FnArg>) -> Result<(&Type, String)> {
    let arg =
        arg.ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "Missing message argument"))?;
    if let FnArg::Typed(PatType { ty, .. }) = arg {
        Ok((ty.as_ref(), type_to_string(ty)))
    } else {
        Err(Error::new(
            arg.span(),
            "Expected a typed argument for message",
        ))
    }
}

fn extract_actor_from_mut(arg: Option<&FnArg>) -> Result<(&Type, String)> {
    let arg =
        arg.ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "Missing actor argument"))?;
    if let FnArg::Typed(PatType { ty, .. }) = arg {
        if let Type::Reference(tr) = ty.as_ref() {
            if tr.mutability.is_some() {
                let inner = tr.elem.as_ref();
                return Ok((inner, type_to_string(inner)));
            }
        }
    }
    Err(Error::new(
        arg.span(),
        "First argument of a sync handler must be '&mut ActorType'",
    ))
}

fn extract_actor_from_async_ctx(arg: Option<&FnArg>) -> Result<(&Type, String)> {
    let arg = arg.ok_or_else(|| {
        Error::new(
            proc_macro2::Span::call_site(),
            "Missing AsyncContext argument",
        )
    })?;
    if let FnArg::Typed(PatType { ty, .. }) = arg {
        if let Type::Path(tp) = ty.as_ref() {
            let last_segment = tp
                .path
                .segments
                .last()
                .ok_or_else(|| Error::new(arg.span(), "Invalid path for AsyncContext"))?;

            if last_segment.ident == "AsyncContext" {
                if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Ok((inner_ty, type_to_string(inner_ty)));
                    }
                }
                return Err(Error::new(
                    arg.span(),
                    "AsyncContext must specify the Actor type in angle brackets: AsyncContext<MyActor>",
                ));
            }
        }
    }
    Err(Error::new(
        arg.span(),
        "First argument of an async handler must be 'AsyncContext<ActorType>'",
    ))
}
