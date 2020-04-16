//! Internal implementation details of `xtensa-lx6-rt`.
//!
//! Do not use this crate directly.

#![deny(warnings)]
#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
//use std::collections::HashSet;
use darling::FromMeta;
use syn::spanned::Spanned;
use syn::{parse, parse_macro_input, AttributeArgs, Item};

// check if all called function are in ram
// check if all used data i in ram
// check that no constants are use in teh function (these cannot be forced to ram)
fn check_ram_function(_func: &syn::ItemFn) {
    //    eprintln!("{:?}", func);
}

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct RamArgs {
    rtc_fast: bool,
    rtc_slow: bool,
    uninitialized: bool,
}

/// #[ram] attribute allows placing statics, constants and functions into ram
///

#[proc_macro_attribute]
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let RamArgs {
        rtc_fast,
        rtc_slow,
        uninitialized,
    } = match FromMeta::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    if rtc_slow && rtc_fast {
        return parse::Error::new(
            Span::call_site(),
            "Only one of rtc_slow and rtc_fast is allowed",
        )
        .to_compile_error()
        .into();
    }

    let (section_name_data, section_name_text) = if rtc_slow {
        (
            if uninitialized {
                ".rtc_slow.noinit"
            } else {
                ".rtc_slow.data"
            },
            ".rtc_slow.text",
        )
    } else if rtc_fast {
        (
            if uninitialized {
                ".rtc_fast.noinit"
            } else {
                ".rtc_fast.data"
            },
            ".rtc_fast.text",
        )
    } else {
        (if uninitialized { ".noinit" } else { ".data" }, ".rwtext")
    };

    let item: syn::Item = syn::parse(input).expect("failed to parse input");

    let section: proc_macro2::TokenStream;
    match item {
        Item::Static(ref _struct_item) => section = quote! {#[link_section=#section_name_data]},
        Item::Const(ref _struct_item) => section = quote! {#[link_section=#section_name_data]},
        Item::Fn(ref function_item) => {
            check_ram_function(function_item);
            section = quote! {#[link_section=#section_name_text]};
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
}
