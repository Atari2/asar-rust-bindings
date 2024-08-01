use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// This macro is used to ensure that the global lock is used in the function.
/// 
/// Usage 
/// ```ignore
/// use asar_snes::use_asar_global_lock;
/// 
/// #[use_asar_global_lock]
/// fn my_function() {
///    // code that uses multiple asar api calls in a multithreaded environment
/// }
/// ```
#[proc_macro_attribute]
pub fn use_asar_global_lock(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_return_type = &input_fn.sig.output;
    let fn_arguments = &input_fn.sig.inputs;
    let fn_visilibility = &input_fn.vis;
    let doc_comments = &input_fn.attrs;

    let expanded = quote! {
        #(#doc_comments)*
        #fn_visilibility fn #fn_name(#fn_arguments) #fn_return_type {
            crate::with_asar_lock(|| {
                #fn_block
            })
        }
    };

    TokenStream::from(expanded)
}
