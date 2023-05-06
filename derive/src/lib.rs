extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

#[proc_macro_derive(TomlExample)]
#[proc_macro_error]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;

    let output = quote! {
        impl toml_example::TomlExample for #struct_name {
            fn to_example() -> String {
                format!("# Toml example for {}", stringify!(#struct_name))
            }
        }
    };
    TokenStream::from(output)
}
