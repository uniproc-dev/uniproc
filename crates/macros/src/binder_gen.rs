use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemTrait, PathArguments, TraitItem, Type, TypeParamBound, WherePredicate};

pub fn generate_binder(trait_item: &ItemTrait) -> TokenStream {
    let trait_ident = &trait_item.ident;
    let base_name = trait_ident
        .to_string()
        .replace("Ui", "")
        .replace("Bindings", "");

    let binder_name = format_ident!("{}Binder", base_name);
    let partial_binder_name = format_ident!("{}PartialBinder", base_name);

    let methods: Vec<_> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let TraitItem::Fn(m) = item {
                Some(m)
            } else {
                None
            }
        })
        .collect();

    let state_params: Vec<_> = methods
        .iter()
        .map(|m| format_ident!("S_{}", m.sig.ident))
        .collect();

    let all_empty: Vec<_> = methods.iter().map(|_| quote! { () }).collect();
    let all_done: Vec<_> = methods.iter().map(|_| quote! { Done }).collect();

    let method_impls = methods.iter().enumerate().map(|(i, method)| {
        let method_ident = &method.sig.ident;
        let (arity, types) = extract_handler_types(method);

        let mut impl_params = state_params.clone();
        impl_params.remove(i);

        let mut state_before = state_params
            .iter()
            .map(|p| quote! { #p })
            .collect::<Vec<_>>();
        state_before[i] = quote! { () };

        let mut state_after = state_params
            .iter()
            .map(|p| quote! { #p })
            .collect::<Vec<_>>();
        state_after[i] = quote! { Done };

        let inner_call = format_ident!("on{}", arity);

        let from_tuple = if arity == 0 {
            quote!(())
        } else if arity == 1 {
            let ty = &types[0];
            quote!(#ty)
        } else {
            quote!((#(#types),*))
        };

        let inner_args = if arity == 0 {
            quote! { M::from(()) }
        } else if arity == 1 {
            quote! { |arg0| M::from(arg0) }
        } else {
            let arg_ids: Vec<_> = (0..arity).map(|idx| format_ident!("arg{}", idx)).collect();
            quote! { |#(#arg_ids),*| M::from((#(#arg_ids),*)) }
        };

        quote! {

            #[allow(non_camel_case_types)]
            impl<'p, A, P, #(#impl_params),*> #binder_name<'p, A, P, #(#state_before),*>
            where A: 'static, P: #trait_ident, #(#impl_params: 'static),*
            {
                pub fn #method_ident<M>(self) -> #binder_name<'p, A, P, #(#state_after),*>
                where
                    M: app_core::actor::Message + Clone + std::convert::From<#from_tuple>,
                    A: app_core::actor::Handler<M>
                {
                    #binder_name {
                        inner: self.inner.#inner_call(|p, f| p.#method_ident(f), #inner_args),
                        _states: std::marker::PhantomData,
                    }
                }
            }

            #[allow(non_camel_case_types)]
            impl<'p, A, P> #partial_binder_name<'p, A, P>
            where A: 'static, P: #trait_ident
            {
                pub fn #method_ident<M>(mut self) -> Self
                where
                    M: app_core::actor::Message + Clone + std::convert::From<#from_tuple>,
                    A: app_core::actor::Handler<M>
                {
                    self.inner = self.inner.#inner_call(|p, f| p.#method_ident(f), #inner_args);
                    self
                }
            }
        }
    });

    quote! {
        pub struct Done;

        #[allow(non_camel_case_types)]
        pub struct #binder_name<'p, A: 'static, P, #(#state_params),*> {
            inner: app_core::actor::binder::UiBinder<'p, A, P>,
            _states: std::marker::PhantomData<(#(#state_params),*)>,
        }

        #[allow(non_camel_case_types)]
        impl<'p, A: 'static, P: #trait_ident> #binder_name<'p, A, P, #(#all_empty),*> {
            pub fn new(addr: &app_core::actor::Addr<A>, port: &'p P) -> Self {
                Self {
                    inner: app_core::actor::binder::UiBinder::new(addr, port),
                    _states: std::marker::PhantomData,
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<'p, A: 'static, P: #trait_ident> #binder_name<'p, A, P, #(#all_done),*> {
            pub fn build(self) -> app_core::actor::binder::UiBinder<'p, A, P> {
                self.inner
            }
        }

        pub struct #partial_binder_name<'p, A: 'static, P> {
            inner: app_core::actor::binder::UiBinder<'p, A, P>,
        }

        #[allow(non_camel_case_types)]
        impl<'p, A: 'static, P: #trait_ident> #partial_binder_name<'p, A, P> {
            pub fn new(addr: &app_core::actor::Addr<A>, port: &'p P) -> Self {
                Self {
                    inner: app_core::actor::binder::UiBinder::new(addr, port),
                }
            }

            pub fn build(self) -> app_core::actor::binder::UiBinder<'p, A, P> {
                self.inner
            }
        }

        #(#method_impls)*
    }
}

fn extract_handler_types(method: &syn::TraitItemFn) -> (usize, Vec<Type>) {
    let mut types = Vec::new();
    let Some(where_clause) = &method.sig.generics.where_clause else {
        return (0, types);
    };
    for predicate in &where_clause.predicates {
        if let WherePredicate::Type(pred) = predicate {
            for bound in &pred.bounds {
                if let TypeParamBound::Trait(tr) = bound {
                    let segment = tr.path.segments.last().unwrap();
                    if ["Fn", "FnMut", "FnOnce"].contains(&segment.ident.to_string().as_str()) {
                        if let PathArguments::Parenthesized(args) = &segment.arguments {
                            for input_ty in &args.inputs {
                                types.push(input_ty.clone());
                            }
                            return (types.len(), types);
                        }
                    }
                }
            }
        }
    }
    (0, types)
}
