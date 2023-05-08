extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments,
    AttrStyle::Outer,
    Attribute,
    Expr::Lit,
    ExprLit, Field,
    Fields::Named,
    GenericArgument,
    Lit::Str,
    Meta::{List, NameValue},
    MetaList, MetaNameValue, PathArguments, PathSegment, Type, TypePath,
};

enum DefaultSource {
    DefaultValue(String),
    DefaultFn(Option<String>),
    SerdeDefaultFn(String),
}

fn default_value(ty: String) -> String {
    match ty.as_str() {
        "usize" | "u8" | "u16" | "u32" | "u64" | "u128" | "isize" | "i8" | "i16" | "i32"
        | "i64" | "i128" => "0",
        "f32" | "f64" => "0.0",
        _ => "\"\"",
    }
    .to_string()
}

fn parse_type(ty: &Type, default: &mut String, optional: &mut bool) -> Option<String> {
    let mut r#type = None;
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            let id = ident.to_string();
            if arguments.is_none() {
                r#type = Some(id.clone());
                *default = default_value(id);
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
                        let mut item_default_value = String::new();
                        parse_type(&ty, &mut item_default_value, &mut false);
                        *default = if item_default_value.is_empty() {
                            format!("[  ]")
                        } else {
                            format!("[ {item_default_value:}, ]")
                        }
                    }
                }
            }
            // TODO else Complex struct in else
        }
    }
    r#type
}

fn parse_doc_default_attrs(attrs: &Vec<Attribute>) -> (Vec<String>, Option<DefaultSource>) {
    let mut docs = Vec::new();
    let mut default_source = None;
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
            (Outer, List(MetaList { path, tokens, .. }))
                if path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string() == "serde")
                    .unwrap_or_default()
                    == true =>
            {
                let token_str = tokens.to_string();
                if token_str.starts_with("default") {
                    if let Some(s) = token_str.split_once(" = ") {
                        default_source =
                            Some(DefaultSource::SerdeDefaultFn(s.1.trim_matches('"').into()));
                    } else {
                        default_source = Some(DefaultSource::DefaultFn(None));
                    }
                }
            }
            _ => (),
        }
    }
    (docs, default_source)
}

fn get_default_and_doc_from_field(field: &Field) -> (DefaultSource, Vec<String>, bool) {
    let mut default_value = String::new();
    let mut optional = false;
    let (docs, default_source) = parse_doc_default_attrs(&field.attrs);
    let ty = parse_type(&field.ty, &mut default_value, &mut optional);
    let default_source  = match default_source {
        Some(DefaultSource::DefaultFn(_)) => DefaultSource::DefaultFn(ty),
        Some(DefaultSource::SerdeDefaultFn(f)) => DefaultSource::SerdeDefaultFn(f),
        _ => DefaultSource::DefaultValue(default_value),
    };
    (
        default_source,
        docs,
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
    let mut example = "\"".to_string();
    push_doc_string(&mut example, parse_doc_default_attrs(&input.attrs).0, true);

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
                match default {
                    DefaultSource::DefaultValue(default) => {
                        example.push_str(&field_name);
                        example.push_str(" = ");
                        example.push_str(&default.replace("\\", "\\\\").replace("\"", "\\\""));
                        example.push('\n');
                    }
                    DefaultSource::DefaultFn(None) => {
                        example.push_str(&field_name);
                        example.push_str(" = \"\"\n");
                    }
                    DefaultSource::DefaultFn(Some(ty)) => {
                        example.push_str(&field_name);
                        example.push_str(" = \".to_string()");
                        example.push_str(&format!(" + &format!(\"{{:?}}\",  {ty}::default())"));
                        example.push_str(" + &\"\n");
                    }
                    DefaultSource::SerdeDefaultFn(fn_str) => {
                        example.push_str(&field_name);
                        example.push_str(" = \".to_string()");
                        example.push_str(&format!(" + &format!(\"{{:?}}\",  {fn_str}())"));
                        example.push_str(&"+ &\"\n");
                    }
                }
            }
        }
    }
    example.push_str("\".to_string()");

    let stream: proc_macro2::TokenStream = example.parse().unwrap();

    let output = quote! {
        impl toml_example::TomlExample for #struct_name {
            fn toml_example() -> String {
                #stream
            }
        }
    };
    TokenStream::from(output)
}
