extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Attribute for wrapping a function in a child span.
/// The span is created from the current context and closed at the end of the function.
#[proc_macro_attribute]
pub fn with_otel_span(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_ident = &input.sig.ident;
    let func_block = &input.block;
    let func_vis = &input.vis;
    let func_sig = &input.sig;

    let expanded = quote! {
        #func_vis #func_sig {
            let (otel_ctx, otel_span) = otelx_core::new_child_span(stringify!(#func_ident), None);

            let _guard = otelx_core::set_context(otel_ctx.clone());

            let result = { async #func_block }.await;
            otel_span.end();
            result
        }
    };
    TokenStream::from(expanded)
}
