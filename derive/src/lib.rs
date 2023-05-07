extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, AttrStyle::Outer, Attribute, Expr::Lit, ExprLit, Field,
    Fields::Named, GenericArgument, Lit::Str, Meta::NameValue, MetaNameValue, PathArguments,
    PathSegment, Type, TypePath,
};

fn default_value(ty: String) -> String {
    match ty.as_str() {
        "usize" | "u8" | "u16" | "u32" | "u64" | "u128" | "isize" | "i8" | "i16" | "i32"
        | "i64" | "i128" => "0",
        "f32" | "f64" => "0.0",
        _ => "\"\"",
    }
    .to_string()
}

fn parse_type(ty: &Type, default: &mut Option<String>, optional: &mut bool) {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            let id = ident.to_string();
            if arguments.is_none() {
                *default = Some(default_value(id));
            } else if id == "Option" {
                *optional = true;
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.first() {
                        parse_type(&ty, default, &mut false);
                    }
                }
            } else if id == "Vec" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.first() {
                        let mut item_default = None;
                        parse_type(&ty, &mut item_default, &mut false);
                        *default = if let Some(item_default) = item_default {
                            Some(format!("[ {item_default:}, ]"))
                        } else {
                            Some(format!("[  ]"))
                        }
                    }
                }
            }
            // TODO else Complex struct in else
        }
    }
}

fn parse_docs(attrs: &Vec<Attribute>) -> Vec<String> {
    let mut docs = Vec::new();
    for attr in attrs.iter() {
        match (attr.style, &attr.meta) {
            (Outer, NameValue(MetaNameValue { path, value, .. })) => {
                for seg in path.segments.iter() {
                    if seg.ident.to_string() == "doc" {
                        if let Lit(ExprLit {
                            lit: Str(lit_str), ..
                        }) = value
                        {
                            docs.push(lit_str.value());
                        }
                    }
                }
            }
            _ => (),
        }
    }
    docs
}

fn get_default_and_doc_from_field(field: &Field) -> (Option<String>, Vec<String>, bool) {
    let mut default = None;
    let mut optional = false;
    parse_type(&field.ty, &mut default, &mut optional);
    (
        default.map(|s| s.to_string()),
        parse_docs(&field.attrs),
        optional,
    )
}

fn push_doc_string(example: &mut String, docs: Vec<String>, paragraph: bool) {
    let has_docs = !docs.is_empty();
    for doc in docs.into_iter() {
        example.push('#');
        example.push_str(&doc);
        example.push('\n');
    }

    if has_docs && paragraph {
        example.push('\n');
    }
}

#[proc_macro_derive(TomlExample)]
#[proc_macro_error]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut example = String::new();
    push_doc_string(&mut example, parse_docs(&input.attrs), true);

    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
    } else {
        abort!(&input.ident, "TomlExample derive only use for struct")
    };
    if let Named(fields_named) = fields {
        for f in fields_named.named.iter() {
            if let Some(field_name) = f.ident.as_ref().map(|i| i.to_string()) {
                let (default, doc_str, optional) = get_default_and_doc_from_field(&f);
                push_doc_string(&mut example, doc_str, false);

                if optional {
                    example.push_str("# ");
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
