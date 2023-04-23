/* Copyright (c) 2022 Benjamin John Mordaunt
 *     The OpenNitro Project
 */
use proc_macro::{TokenStream, Span};
use syn::token::{Paren, Token};
use std::collections::HashSet as Set;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident, Block, Attribute, Token, LitStr, GenericArgument, parenthesized, Expr, LitInt, ExprLit, ExprCall, ExprPath, Path, parse_quote};
use syn::parse::{Parse, ParseStream, Result, ParseBuffer};
use syn::punctuated::Punctuated;

struct Args {
    vars: Set<String>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter()
                .map(|ref x| x.to_string())
                .collect()
        })
    }
}

#[proc_macro_attribute]
pub fn bios_call(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut orig_fn = parse_macro_input!(input as ItemFn);
    let mut shim_func = orig_fn.clone();

    let is_no_mangle = {
        let mut found = false;
        for attr in &orig_fn.attrs {
            if attr.meta.path().is_ident("no_mangle") {
                found = true;
                break;
            }
        }
        found
    };
    assert!(is_no_mangle, "check_callsite fns must also be no_mangle");

    /* Some functions accept 64-bit arguments via a single register in a shim.
       The MSB is passed in r1 and the LSB in r2 */
    let args = parse_macro_input!(args as Args);

    let expand64 = args.vars.contains("expand64");
    let expand64_or_empty: Punctuated<LitStr, Token![,]> =  
    {
        if expand64 {
            syn::parse_quote! {
                "mov r2, r1",
                "add r1, r1, #4",
            }
        } else {
            syn::parse_quote! { }
        }
    };

    let new_orig_fn_ident = Ident::new(
        &format!("RAW_{}", orig_fn.sig.ident), 
        orig_fn.sig.ident.span()
    );

    orig_fn.sig.ident = new_orig_fn_ident.clone();

    let naked_attribute: Attribute = syn::parse_quote! { #[naked] };
    shim_func.attrs.push(naked_attribute);

    let arm_inst_set: Attribute = syn::parse_quote! { #[instruction_set(arm::a32)] };
    shim_func.attrs.push(arm_inst_set);


    let shim_body: Block = syn::parse_quote! {{
        unsafe {
            asm!(
                #expand64_or_empty
                "ldr ip, ={tgt}",
                "b {biossafe}",
                tgt = sym #new_orig_fn_ident,
                biossafe = sym BiosSafeShim,
                options(noreturn)
            );
        }
    }};

    shim_func.block = Box::new(shim_body);

    let output_fn = quote! { 
        #orig_fn
        #shim_func
    };
    output_fn.into()
}

#[proc_macro]
pub fn with_shim(input: TokenStream) -> TokenStream {
    let mut args_pb = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated).into_iter();
    let shimidx = match args_pb.next() {
        Some(Expr::Lit(x)) => {
            match x.lit {
                syn::Lit::Int(x) => x,
                _ => panic!("Shim index literal not an integer"),
            }
        },
        _ => panic!("Shim index not present or invalid"),
    };
    let fnname = match args_pb.next() {
        Some(Expr::Path(x)) => {
            if let Some(ident) = x.path.get_ident() {
                let mut new_path: ExprPath = x.clone();
                let new_ident = Ident::new(&format!("SHIM{}_{}", shimidx.base10_parse::<u32>().unwrap(), ident), ident.span());
                new_path.path = new_ident.into();
                new_path
            } else {
                panic!("Shim underlying function not valid identifier")
            }
        },
        _ => panic!("Shim function name argument invalid")
    };
    let fn_args: Punctuated::<Expr, Token![,]> = args_pb.collect();

    let fn_call = Expr::Call(ExprCall { 
        attrs: vec![], 
        func: Box::new(Expr::Path(fnname)), 
        paren_token: Paren::default(), 
        args: fn_args 
    });

    quote::quote! {
        #fn_call
    }.into()
}