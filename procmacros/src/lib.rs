//! Internal implementation details of `xtensa-lx6-rt`.
//!
//! Do not use this crate directly.
//!
//! # TODO:
//! [ ] Checking of all called functions and data are in ram
//! [ ] Automatic checking of 0 init and then use .bss segment

#![deny(warnings)]
#![allow(unused_braces)]
#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::Span;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::iter;
use syn::{
    parse, parse_macro_input, spanned::Spanned, AttrStyle, Attribute, AttributeArgs, FnArg, Ident,
    Item, ItemFn, ItemStatic, Meta::Path, ReturnType, Stmt, Type, Visibility,
};

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
        Item::Const(ref _struct_item) => section = quote! {#[link_section=#section_name_data]},
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

/// Marks a function as an interrupt handler
///
/// When specified between braces that interrupt will be used and the function
/// can have an arbitrary name. Otherwise the name of the function must be the name of the
/// interrupt.
#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f: ItemFn = syn::parse(input).expect("`#[interrupt]` must be applied to a function");

    let attr_args = parse_macro_input!(args as AttributeArgs);

    if attr_args.len() > 1 {
        Span::call_site()
            .error("This attribute accepts zero or 1 arguments")
            .emit();
    }

    let ident = f.sig.ident.clone();
    let mut ident_s = &ident.clone();

    if attr_args.len() == 1 {
        match &attr_args[0] {
            syn::NestedMeta::Meta(Path(x)) => {
                ident_s = x.get_ident().unwrap();
            }
            _ => {
                Span::call_site()
                    .error(format!(
                        "This attribute accepts a string attribute {:?}",
                        attr_args[0]
                    ))
                    .emit();
            }
        }
    }

    // XXX should we blacklist other attributes?

    if let Err(error) = check_attr_whitelist(&f.attrs, WhiteListCaller::Interrupt) {
        return error;
    }

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                Type::Never(..) => true,
                _ => false,
            },
        };

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[interrupt]` handlers must have signature `[unsafe] fn() [-> !]`",
        )
        .to_compile_error()
        .into();
    }

    let (statics, stmts) = match extract_static_muts(f.block.stmts.iter().cloned()) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    f.sig.ident = Ident::new(
        &format!("__xtensa_lx_6_{}", f.sig.ident),
        proc_macro2::Span::call_site(),
    );
    f.sig.inputs.extend(statics.iter().map(|statik| {
        let ident = &statik.ident;
        let ty = &statik.ty;
        let attrs = &statik.attrs;
        syn::parse::<FnArg>(quote!(#[allow(non_snake_case)] #(#attrs)* #ident: &mut #ty).into())
            .unwrap()
    }));
    f.block.stmts = iter::once(
        syn::parse2(quote! {{
            // Check that this interrupt actually exists
            interrupt::#ident_s;
        }})
        .unwrap(),
    )
    .chain(stmts)
    .collect();

    let tramp_ident = Ident::new(
        &format!("{}_trampoline", f.sig.ident),
        proc_macro2::Span::call_site(),
    );
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

    let (ref cfgs, ref attrs) = extract_cfgs(f.attrs.clone());

    let export_name = ident_s.to_string();

    quote!(
        #(#cfgs)*
        #(#attrs)*
        #[doc(hidden)]
        #[export_name = #export_name]
        pub unsafe extern "C" fn #tramp_ident() {
            #ident(
                #(#resource_args),*
            )
        }

        #[inline(always)]
        #f
    )
    .into()
}

/// Extracts `static mut` vars from the beginning of the given statements
fn extract_static_muts(
    stmts: impl IntoIterator<Item = Stmt>,
) -> Result<(Vec<ItemStatic>, Vec<Stmt>), parse::Error> {
    let mut istmts = stmts.into_iter();

    let mut seen = HashSet::new();
    let mut statics = vec![];
    let mut stmts = vec![];
    while let Some(stmt) = istmts.next() {
        match stmt {
            Stmt::Item(Item::Static(var)) => {
                if var.mutability.is_some() {
                    if seen.contains(&var.ident) {
                        return Err(parse::Error::new(
                            var.ident.span(),
                            format!("the name `{}` is defined multiple times", var.ident),
                        ));
                    }

                    seen.insert(var.ident.clone());
                    statics.push(var);
                } else {
                    stmts.push(Stmt::Item(Item::Static(var)));
                }
            }
            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    Ok((statics, stmts))
}

fn extract_cfgs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>) {
    let mut cfgs = vec![];
    let mut not_cfgs = vec![];

    for attr in attrs {
        if eq(&attr, "cfg") {
            cfgs.push(attr);
        } else {
            not_cfgs.push(attr);
        }
    }

    (cfgs, not_cfgs)
}

enum WhiteListCaller {
    Interrupt,
}

fn check_attr_whitelist(attrs: &[Attribute], caller: WhiteListCaller) -> Result<(), TokenStream> {
    let whitelist = &[
        "doc",
        "link_section",
        "cfg",
        "allow",
        "warn",
        "deny",
        "forbid",
        "cold",
        "ram",
    ];

    'o: for attr in attrs {
        for val in whitelist {
            if eq(&attr, &val) {
                continue 'o;
            }
        }

        let err_str = match caller {
            WhiteListCaller::Interrupt => {
                "this attribute is not allowed on an interrupt handler controlled by esp32_hal"
            }
        };

        return Err(parse::Error::new(attr.span(), &err_str)
            .to_compile_error()
            .into());
    }

    Ok(())
}

/// Returns `true` if `attr.path` matches `name`
fn eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path.is_ident(name)
}
