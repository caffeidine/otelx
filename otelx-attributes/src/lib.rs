use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    Meta, Token,
    parse::{Parse, ParseStream},
};

struct Args(Punctuated<Meta, Token![,]>);

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Args(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro_attribute]
pub fn tracing(args: TokenStream, input: TokenStream) -> TokenStream {
    // Analyse des arguments de l'attribut
    let Args(args) = syn::parse_macro_input!(args as Args);
    let mut semantic = None;
    let mut span_name = None;
    for meta in args.iter() {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("semantic") {
                semantic = Some(nv.value.clone());
            } else if nv.path.is_ident("span_name") {
                span_name = Some(nv.value.clone());
            }
        }
    }
    let semantic = semantic.expect("semantic parameter is required");
    let span_name = span_name.expect("span_name parameter is required");

    let input_fn = syn::parse_macro_input!(input as syn::ItemFn);
    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let mut param_inserts = Vec::new();
    for arg in sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let ident = &pat_ident.ident;
                let name = ident.to_string();
                param_inserts.push(quote! {
                    map.insert(#name, format!("{:?}", #ident));
                });
            }
        }
    }
    let param_capture = quote! {
        let __otel_params: ::std::collections::HashMap<&'static str, String> = {
            let mut map = ::std::collections::HashMap::new();
            #(#param_inserts)*
            map
        };
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #param_capture
            otelx::trace_with_adapter(
                #semantic,
                #span_name,
                async move {
                    let __res = async move #block .await;
                    __res
                },
                __otel_params
            ).await.into()
        }
    };

    TokenStream::from(expanded)
}
