//! Internal implementation details of `xtensa-lx6-rt`.
//!
//! Do not use this crate directly.
//!
//! # TODO:
//! [ ] Checking of all called functions and data are in ram
//! [ ] Automatic checking of 0 init and then use .bss segment

#![deny(warnings)]
#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::Span;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Item};

// TODO:
// - check if all called function are in ram
// - check if all used data is in ram
// - check that no constants are use in the function (these cannot be forced to ram)
fn check_ram_function(_func: &syn::ItemFn) {
    //    eprintln!("{:?}", func);
}

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct RamArgs {
    rtc_fast: bool,
    rtc_slow: bool,
    external: bool,
    uninitialized: bool,
    zeroed: bool,
}

/// This attribute allows placing statics, constants and functions into ram.
///
/// Options that can be specified are rtc_slow, rtc_fast or external to use the
/// RTC slow, RTC fast ram or external SPI RAM instead of the normal SRAM.
///
/// The uninitialized option will skip initialization of the memory
/// (e.g. to persist it across resets or deep sleep mode for the RTC RAM)

#[proc_macro_attribute]
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let RamArgs {
        rtc_fast,
        rtc_slow,
        external,
        uninitialized,
        zeroed,
    } = match FromMeta::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    if rtc_slow && rtc_fast {
        Span::call_site()
            .error("Only one of rtc_slow and rtc_fast is allowed")
            .emit();
    }

    if rtc_slow && rtc_fast {
        Span::call_site()
            .error("Only one of uninitialized and zeroed")
            .emit();
    }

    if external && cfg!(not(feature = "external_ram")) {
        Span::call_site()
            .error("External ram support not enabled")
            .emit();
    }

    let section_name_data = if rtc_slow {
        if uninitialized {
            ".rtc_slow.noinit"
        } else if zeroed {
            ".rtc_slow.bss"
        } else {
            ".rtc_slow.data"
        }
    } else if rtc_fast {
        if uninitialized {
            ".rtc_fast.noinit"
        } else if zeroed {
            ".rtc_fast.bss"
        } else {
            ".rtc_fast.data"
        }
    } else if external {
        if uninitialized {
            ".external.noinit"
        } else if zeroed {
            ".external.bss"
        } else {
            ".external.data"
        }
    } else {
        if uninitialized {
            ".noinit"
        } else {
            ".data"
        }
    };

    let section_name_text = if rtc_slow {
        ".rtc_slow.text"
    } else if rtc_fast {
        ".rtc_fast.text"
    } else if external {
        ".invalid"
    } else {
        ".rwtext"
    };

    let item: syn::Item = syn::parse(input).expect("failed to parse input");

    let section: proc_macro2::TokenStream;
    match item {
        Item::Static(ref _struct_item) => section = quote! {#[link_section=#section_name_data]},
        Item::Fn(ref function_item) => {
            if zeroed {
                Span::call_site()
                    .error("Zeroed is not applicable to functions")
                    .emit();
            }
            if uninitialized {
                Span::call_site()
                    .error("Uninitialized is not applicable to functions")
                    .emit();
            }
            if external {
                Span::call_site()
                    .error("External is not applicable to functions")
                    .emit();
            }
            check_ram_function(function_item);
            section = quote! {
                #[link_section=#section_name_text]
                #[inline(never)] // make certain function is not inlined
            };
        }
        _ => {
            section = quote! {};
            item.span()
                .unstable()
                .error("#[ram] attribute can only be applied to functions and statics")
                .emit();
        }
    }

    let output = quote! {
        #section
        #item
    };
    output.into()
}
