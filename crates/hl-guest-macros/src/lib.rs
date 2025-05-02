use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned as _;
use syn::{parse_macro_input, parse_quote, ForeignItemFn, ItemFn, LitStr, Pat};

enum NameArg {
    None,
    Name(LitStr),
}

impl Parse for NameArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(NameArg::None);
        }
        let name: LitStr = input.parse()?;
        if !input.is_empty() {
            return Err(Error::new(input.span(), "expected a single identifier"));
        }
        Ok(NameArg::Name(name))
    }
}

#[proc_macro_attribute]
pub fn guest_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_name = crate_name("hl-guest").expect("hl-guest must be a dependency");
    let crate_name = match crate_name {
        FoundCrate::Itself => quote! {crate},
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! {::#ident}
        }
    };

    let fn_declaration = parse_macro_input!(item as ItemFn);

    let ident = fn_declaration.sig.ident.clone();

    let exported_name = match parse_macro_input!(attr as NameArg) {
        NameArg::None => quote! { stringify!(#ident) },
        NameArg::Name(name) => quote! { #name },
    };

    let mut args = vec![];
    let mut args_names = vec![];
    for arg in fn_declaration.sig.inputs.iter() {
        match arg {
            syn::FnArg::Receiver(_) => {
                return Error::new(
                    arg.span(),
                    "Receiver (self) argument is not allowed in guest functions",
                )
                .to_compile_error()
                .into();
            }
            syn::FnArg::Typed(arg) => {
                let ty = &arg.ty;
                args.push(quote! { #ty });

                let pat = &arg.pat;
                args_names.push(quote! { #pat });
            }
        }
    }

    let ret = match &fn_declaration.sig.output {
        syn::ReturnType::Default => quote! { quote! { () } },
        syn::ReturnType::Type(_, ty) => {
            quote! { #ty }
        }
    };

    let mut fn_declaration = fn_declaration;
    if fn_declaration.sig.asyncness.is_some() {
        fn_declaration.sig.asyncness = None;
        let block = fn_declaration.block.clone();
        fn_declaration.block = Box::new(syn::Block {
            brace_token: fn_declaration.block.brace_token,
            stmts: parse_quote! {
                hl_guest_async::block_on(async move {
                    #block
                })
            },
        });
    }

    let output = quote! {
        #fn_declaration

        mod #ident {
            use super::*;

            #[#crate_name::__private::linkme::distributed_slice(#crate_name::__private::GUEST_FUNCTION_INIT)]
            #[linkme(crate = #crate_name::__private::linkme)]
            static REGISTRATION: fn() = registration;

            pub fn registration() {
                use #crate_name::__private::alloc::{vec, format};
                use #crate_name::__private::alloc::string::ToString as _;
                use #crate_name::__private::alloc::vec::Vec;

                use #crate_name::__private::hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;
                use #crate_name::__private::hyperlight_common::flatbuffer_wrappers::function_types::ParameterValue;
                use #crate_name::__private::hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
                use #crate_name::__private::hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result;

                use #crate_name::__private::hyperlight_guest::error::HyperlightGuestError;
                use #crate_name::__private::hyperlight_guest::guest_function_definition::GuestFunctionDefinition;
                use #crate_name::__private::hyperlight_guest::guest_function_register::register_function;

                fn wrapper(function_call: &FunctionCall) -> ::core::result::Result<Vec<u8>, HyperlightGuestError> {
                    static EMPTY_VEC: Vec<ParameterValue> = vec![];
                    let mut parameters = function_call.parameters.as_ref().unwrap_or(&EMPTY_VEC).iter().cloned();
                    let ret = super::#ident(
                        #(
                            <#args as #crate_name::__private::ty::ToFlatbufParameter>::from_value(
                                parameters.next().ok_or(
                                    HyperlightGuestError::new(
                                        ErrorCode::GuestFunctionParameterTypeMismatch,
                                        format!("Missing parameter {}", stringify!(#args_names)),
                                    )
                                )?
                            )?
                        ),*
                    );
                    <#ret as #crate_name::__private::ty::IntoFlatbufReturn>::to_value(ret)
                }

                let def = GuestFunctionDefinition::new(
                    #exported_name.to_string(),
                    alloc::vec![
                        #(<#args as #crate_name::__private::ty::ToFlatbufParameter>::TYPE),*
                    ],
                    <#ret as #crate_name::__private::ty::IntoFlatbufReturn>::TYPE,
                    wrapper as usize,
                );

                register_function(def);
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn host_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_name = crate_name("hl-guest").expect("hl-guest must be a dependency");
    let crate_name = match crate_name {
        FoundCrate::Itself => quote! {crate},
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! {::#ident}
        }
    };

    let fn_declaration = parse_macro_input!(item as ForeignItemFn);

    let ForeignItemFn {
        attrs,
        vis,
        sig,
        semi_token: _,
    } = fn_declaration;

    let ident = sig.ident.clone();

    let exported_name = match parse_macro_input!(attr as NameArg) {
        NameArg::None => quote! { stringify!(#ident) },
        NameArg::Name(name) => quote! { #name },
    };

    let mut args = vec![];
    let mut args_names = vec![];
    for arg in sig.inputs.iter() {
        match arg {
            syn::FnArg::Receiver(_) => {
                return Error::new(
                    arg.span(),
                    "Receiver (self) argument is not allowed in guest functions",
                )
                .to_compile_error()
                .into();
            }
            syn::FnArg::Typed(arg) => {
                let ty = &arg.ty;
                args.push(quote! { #ty });

                let Pat::Ident(pat) = *arg.pat.clone() else {
                    return Error::new(
                        arg.span(),
                        "Only named arguments are allowed in host functions",
                    )
                    .to_compile_error()
                    .into();
                };

                if pat.attrs.len() > 0 {
                    return Error::new(
                        arg.span(),
                        "Attributes are not allowed on host function arguments",
                    )
                    .to_compile_error()
                    .into();
                }

                if pat.by_ref.is_some() {
                    return Error::new(
                        arg.span(),
                        "By-ref arguments are not allowed in host functions",
                    )
                    .to_compile_error()
                    .into();
                }

                if pat.mutability.is_some() {
                    return Error::new(
                        arg.span(),
                        "Mutable arguments are not allowed in host functions",
                    )
                    .to_compile_error()
                    .into();
                }

                if pat.subpat.is_some() {
                    return Error::new(
                        arg.span(),
                        "Sub-patterns are not allowed in host functions",
                    )
                    .to_compile_error()
                    .into();
                }

                let ident = pat.ident.clone();

                args_names.push(quote! { #ident });
            }
        }
    }

    let ret = match &sig.output {
        syn::ReturnType::Default => quote! { quote! { () } },
        syn::ReturnType::Type(_, ty) => {
            quote! { #ty }
        }
    };

    let output = quote! {

        #(#attrs)* #vis #sig {
            use #crate_name::__private::alloc::vec;
            use #crate_name::__private::hyperlight_common::flatbuffer_wrappers::function_types::ReturnValue;
            use #crate_name::__private::hyperlight_guest::host_function_call::{get_host_return_value, call_host_function};
            let ret = call_host_function(
                #exported_name,
                Some(vec![
                    #(<#args as #crate_name::__private::ty::ToFlatbufParameter>::to_value(#args_names)),*
                ]),
                <#ret as #crate_name::__private::ty::FromFlatbufReturn>::TYPE,
            );
            <#ret as #crate_name::__private::ty::FromFlatbufReturn>::from_call(ret)
        }

    };

    output.into()
}