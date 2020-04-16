//! Internal implementation details of `xtensa-lx6-rt`.
//!
//! Do not use this crate directly.

#![deny(warnings)]
#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::TokenStream;
//use proc_macro2::Span;
use quote::quote;
//use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::Item;

// check if all called function are in ram
// check if all used data i in ram
// check that no constants are use in teh function (these cannot be forced to ram)
fn check_ram_function(func: &syn::ItemFn) {
    eprintln!("{:?}", func);
}

#[proc_macro_attribute]
pub fn ram(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("failed to parse input");

    let section: proc_macro2::TokenStream;
    match item {
        Item::Static(ref _struct_item) => section = quote! {#[link_section=".data"]},
        Item::Const(ref _struct_item) => section = quote! {#[link_section=".data"]},
        Item::Fn(ref function_item) => {
            check_ram_function(function_item);
            section = quote! {#[link_section=".rwtext"]};
        }
        _ => {
            section = quote! {};
            item.span()
                .unstable()
                .error("#[ram] attribute can only be applied to functions, statics and consts")
                .emit();
        }
    }

    let output = quote! {
        #section
        #item
    };
    output.into()

    //    let f = parse_macro_input!(input as ItemFn);
    /*    TokenStream::from(quote!(
        #[link_section=".rwtext"]
        #f
    ))*/

    /*
        // check the function signature
        let valid_signature = f.sig.constness.is_none()
            && f.vis == Visibility::Inherited
            && f.sig.abi.is_none()
            && f.sig.inputs.is_empty()
            && f.sig.generics.params.is_empty()
            && f.sig.generics.where_clause.is_none()
            && f.sig.variadic.is_none()
            && match f.sig.output {
                ReturnType::Default => false,
                ReturnType::Type(_, ref ty) => match **ty {
                    Type::Never(_) => true,
                    _ => false,
                },
            };

        if !valid_signature {
            return parse::Error::new(
                f.span(),
                "`#[entry]` function must have signature `[unsafe] fn() -> !`",
            )
            .to_compile_error()
            .into();
        }

        if !args.is_empty() {
            return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
                .to_compile_error()
                .into();
        }

        // XXX should we blacklist other attributes?
        let (statics, stmts) = match extract_static_muts(f.block.stmts) {
            Err(e) => return e.to_compile_error().into(),
            Ok(x) => x,
        };

        f.sig.ident = Ident::new(
            &format!("__xtensa_lx6_rt_{}", f.sig.ident),
            Span::call_site(),
        );
        f.sig.inputs.extend(statics.iter().map(|statik| {
            let ident = &statik.ident;
            let ty = &statik.ty;
            let attrs = &statik.attrs;

            // Note that we use an explicit `'static` lifetime for the entry point arguments. This makes
            // it more flexible, and is sound here, since the entry will not be called again, ever.
            syn::parse::<FnArg>(
                quote!(#[allow(non_snake_case)] #(#attrs)* #ident: &'static mut #ty).into(),
            )
            .unwrap()
        }));
        f.block.stmts = stmts;

        let tramp_ident = Ident::new(&format!("{}_trampoline", f.sig.ident), Span::call_site());
        let ident = &f.sig.ident;

        let resource_args = statics
            .iter()
            .map(|statik| {
                let (ref cfgs, ref attrs) = extract_cfgs(statik.attrs.clone());
                let ident = &statik.ident;
                let ty = &statik.ty;
                let expr = &statik.expr;
                quote! {
                    #(#cfgs)*
                    {
                        #(#attrs)*
                        static mut #ident: #ty = #expr;
                        &mut #ident
                    }
                }
            })
            .collect::<Vec<_>>();

        if let Err(error) = check_attr_whitelist(&f.attrs, WhiteListCaller::Entry) {
            return error;
        }

        let (ref cfgs, ref attrs) = extract_cfgs(f.attrs.clone());
        quote!(
            #(#cfgs)*
            #(#attrs)*
            #[doc(hidden)]
            #[export_name = "main"]
            pub unsafe extern "C" fn #tramp_ident() {
                #ident(
                    #(#resource_args),*
                )
            }

            #f
        )
        .into()
    */
}
