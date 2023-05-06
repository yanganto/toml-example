extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    AttrStyle::Outer, Expr::Lit, ExprLit, Field, Fields::Named, Lit::Str, Meta::NameValue,
    MetaNameValue, PathSegment, Type, TypePath,
};

fn get_default_and_doc_from_field(field: &Field) -> (Option<String>, Option<String>) {
    let mut doc = None;
    let mut default = None;
    for attr in field.attrs.iter() {
        match (attr.style, &attr.meta) {
            (Outer, NameValue(MetaNameValue { path, value, .. })) => {
                for seg in path.segments.iter() {
                    if seg.ident.to_string() == "doc" {
                        if let Lit(ExprLit {
                            lit: Str(lit_str), ..
                        }) = value
                        {
                            doc = Some(lit_str.value());
                        }
                    }
                }
            }
            _ => (),
        }
    }
    if let Type::Path(TypePath { path, .. }) = &field.ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            if arguments.is_none() {
                default = match ident.to_string().as_str() {
                    "usize" | "u8" | "u16" | "u32" | "u64" | "u128" | "isize" | "i8" | "i16"
                    | "i32" | "i64" | "i128" => Some("0"),
                    "f32" | "f64" => Some("0.0"),
                    _ => Some("\"\""),
                }
            }
            // TODO Complex struct in else
        }
    }
    (default.map(|s| s.to_string()), doc)
}

#[proc_macro_derive(TomlExample)]
#[proc_macro_error]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut example = format!("# Toml example for {}\n", struct_name.to_string());

    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
    } else {
        abort!(&input.ident, "TomlExample derive only use for struct")
    };
    if let Named(fields_named) = fields {
        for f in fields_named.named.iter() {
            if let Some(field_name) = f.ident.as_ref().map(|i| i.to_string()) {
                let (default, doc_str) = get_default_and_doc_from_field(&f);

                if let Some(doc_str) = doc_str {
                    example.push('#');
                    example.push_str(&doc_str);
                    example.push('\n');
                }

                if let Some(default) = default {
                    example.push_str(&field_name);
                    example.push_str(" = ");
                    example.push_str(&default);
                } else {
                    example.push_str(&field_name);
                    example.push_str(" = \"\"");
                }
                example.push('\n');
            }
        }
    }

    let output = quote! {
        impl toml_example::TomlExample for #struct_name {
            fn toml_example() -> &'static str {
                #example
            }
        }
    };
    TokenStream::from(output)
}
