/* Copyright (c) 2022 Benjamin John Mordaunt
 *     The OpenNitro Project
 */
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident, Block, Attribute};

#[proc_macro_attribute]
pub fn check_callsite(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut orig_fn = parse_macro_input!(input as ItemFn);
    let mut shim_func = orig_fn.clone();

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