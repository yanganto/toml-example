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
mod case;

#[derive(Debug)]
enum DefaultSource {
    DefaultValue(String),
    DefaultFn(Option<String>),
    #[allow(dead_code)]
    SerdeDefaultFn(String),
}

#[derive(PartialEq)]
enum NestingType {
    None,
    Vec,
    Dict,
}

#[derive(PartialEq)]
enum NestingFormat {
    Section(NestingType),
    Prefix,
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

/// return type without Option, Vec
fn parse_type(
    ty: &Type,
    default: &mut String,
    optional: &mut bool,
    nesting_format: &mut Option<NestingFormat>,
) -> Option<String> {
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
                        r#type = parse_type(ty, default, &mut false, nesting_format);
                    }
                }
            } else if id == "Vec" {
                if nesting_format.is_some() {
                    *nesting_format = Some(NestingFormat::Section(NestingType::Vec));
                }
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.first() {
                        let mut item_default_value = String::new();
                        r#type = parse_type(ty, &mut item_default_value, &mut false, &mut None);
                        *default = if item_default_value.is_empty() {
                            "[  ]".to_string()
                        } else {
                            format!("[ {item_default_value:}, ]")
                        }
                    }
                }
            } else if id == "HashMap" || id == "BTreeMap" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.last() {
                        let mut item_default_value = String::new();
                        r#type = parse_type(ty, &mut item_default_value, &mut false, &mut None);
                    }
                }
                if nesting_format.is_some() {
                    *nesting_format = Some(NestingFormat::Section(NestingType::Dict));
                }
            }
            // TODO else Complex struct in else
        }
    }
    r#type
}

#[allow(clippy::type_complexity)]
/// return (doc, default, nesting, require, skip, rename, rename_rule)
fn parse_attrs(
    attrs: &[Attribute],
) -> (
    Vec<String>,
    Option<DefaultSource>,
    Option<NestingFormat>,
    bool,
    bool,
    Option<String>,
    case::RenameRule,
) {
    let mut docs = Vec::new();
    let mut default_source = None;
    let mut nesting_format = None;
    let mut require = false;
    let mut skip = false;
    let mut rename = None;
    let mut rename_rule = case::RenameRule::None;

    for attr in attrs.iter() {
        match (attr.style, &attr.meta) {
            (Outer, NameValue(MetaNameValue { path, value, .. })) => {
                for seg in path.segments.iter() {
                    if seg.ident == "doc" {
                        if let Lit(ExprLit {
                            lit: Str(lit_str), ..
                        }) = value
                        {
                            docs.push(lit_str.value());
                        }
                    }
                }
            }
            (
                Outer,
                List(MetaList {
                    path,
                    tokens: _tokens,
                    ..
                }),
            ) if path
                .segments
                .last()
                .map(|s| s.ident == "serde")
                .unwrap_or_default() =>
            {
                #[cfg(feature = "serde")]
                {
                    let token_str = _tokens.to_string();
                    if token_str.starts_with("default") {
                        if let Some((_, s)) = token_str.split_once('=') {
                            default_source = Some(DefaultSource::SerdeDefaultFn(
                                s.trim().trim_matches('"').into(),
                            ));
                        } else {
                            default_source = Some(DefaultSource::DefaultFn(None));
                        }
                    }
                    if token_str == "skip_deserializing" || token_str == "skip" {
                        skip = true;
                    }
                    if token_str.starts_with("rename") {
                        if token_str.starts_with("rename_all") {
                            if let Some((_, s)) = token_str.split_once('=') {
                                rename_rule = if let Ok(r) =
                                    case::RenameRule::from_str(s.trim().trim_matches('"'))
                                {
                                    r
                                } else {
                                    abort!(&_tokens, "unsupported rename rule")
                                }
                            }
                        } else if let Some((_, s)) = token_str.split_once('=') {
                            rename = Some(s.trim().trim_matches('"').into());
                        }
                    }
                }
            }
            (Outer, List(MetaList { path, tokens, .. }))
                if path
                    .segments
                    .last()
                    .map(|s| s.ident == "toml_example")
                    .unwrap_or_default() =>
            {
                let token_str = tokens.to_string();
                if token_str.starts_with("default") {
                    if let Some((_, s)) = token_str.split_once('=') {
                        default_source = Some(DefaultSource::DefaultValue(s.trim().into()));
                    } else {
                        default_source = Some(DefaultSource::DefaultFn(None));
                    }
                } else if token_str.starts_with("nesting") {
                    if let Some((_, s)) = token_str.split_once('=') {
                        nesting_format = match s.trim() {
                            "prefix" => Some(NestingFormat::Prefix),
                            "section" => Some(NestingFormat::Section(NestingType::None)),
                            _ => abort!(&attr, "please use prefix or section for nesting derive"),
                        }
                    } else {
                        nesting_format = Some(NestingFormat::Section(NestingType::None));
                    }
                } else if token_str == "require" {
                    require = true;
                } else if token_str == "skip" {
                    skip = true;
                } else {
                    abort!(&attr, format!("{} is not allowed attribute", token_str))
                }
            }
            _ => (),
        }
    }
    (
        docs,
        default_source,
        nesting_format,
        require,
        skip,
        rename,
        rename_rule,
    )
}

fn parse_field(
    field: &Field,
) -> (
    DefaultSource,
    Vec<String>,
    bool,
    Option<NestingFormat>,
    bool,
    Option<String>,
) {
    let mut default_value = String::new();
    let mut optional = false;
    let (docs, default_source, mut nesting_format, require, skip, rename, _) =
        parse_attrs(&field.attrs);
    let ty = parse_type(
        &field.ty,
        &mut default_value,
        &mut optional,
        &mut nesting_format,
    );
    let default_source = match default_source {
        Some(DefaultSource::DefaultFn(_)) => DefaultSource::DefaultFn(ty),
        Some(DefaultSource::SerdeDefaultFn(f)) => DefaultSource::SerdeDefaultFn(f),
        Some(DefaultSource::DefaultValue(v)) => DefaultSource::DefaultValue(v),
        _ => DefaultSource::DefaultValue(default_value),
    };
    (
        default_source,
        docs,
        optional && !require,
        nesting_format,
        skip,
        rename,
    )
}

fn push_doc_string(example: &mut String, docs: Vec<String>) {
    for doc in docs.into_iter() {
        example.push('#');
        example.push_str(&doc);
        example.push('\n');
    }
}

fn default_key(default: DefaultSource) -> String {
    if let DefaultSource::DefaultValue(v) = default {
        let key = v.trim_matches('\"').replace(' ', "").replace('.', "-");
        if !key.is_empty() {
            return key;
        }
    }
    "example".into()
}

#[proc_macro_derive(TomlExample, attributes(toml_example))]
#[proc_macro_error]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut struct_doc = "r#\"".to_string();

    // Nesting field example will be append after the non nesting field example to avoid #18
    let mut field_example = "r#\"".to_string();
    let mut nesting_field_example = "".to_string();

    let (doc, _, _, _, _, _, rename_rule) = parse_attrs(&input.attrs);
    push_doc_string(&mut struct_doc, doc);

    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
    } else {
        abort!(&input.ident, "TomlExample derive only use for struct")
    };
    if let Named(fields_named) = fields {
        for f in fields_named.named.iter() {
            let field_type = parse_type(&f.ty, &mut String::new(), &mut false, &mut None);
            if let Some(mut field_name) = f.ident.as_ref().map(|i| i.to_string()) {
                let (default, doc_str, optional, nesting_format, skip, rename) = parse_field(f);
                if skip {
                    continue;
                }
                if let Some(rename) = rename {
                    field_name = rename;
                } else {
                    field_name = rename_rule.apply_to_field(&field_name);
                }
                push_doc_string(&mut field_example, doc_str);

                if nesting_format
                    .as_ref()
                    .map(|f| matches!(f, NestingFormat::Section(_)))
                    .unwrap_or_default()
                {
                    if let Some(field_type) = field_type {
                        nesting_field_example.push_str("\"#.to_string()");
                        let key = default_key(default);
                        match nesting_format {
                            Some(NestingFormat::Section(NestingType::Vec)) if optional => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"# [[{field_name:}]]\n\", \"# \")"
                            )),
                            Some(NestingFormat::Section(NestingType::Vec)) => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"[[{field_name:}]]\n\", \"\")"
                            )),
                            Some(NestingFormat::Section(NestingType::Dict)) if optional => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"# [{field_name:}.{key}]\n\", \"# \")"
                            )),
                            Some(NestingFormat::Section(NestingType::Dict)) => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"[{field_name:}.{key}]\n\", \"\")"
                            )),
                            _ if optional => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"# [{field_name:}]\n\", \"# \")"
                            )),
                            _ => nesting_field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"[{field_name:}]\n\", \"\")"
                            ))
                        };
                        nesting_field_example.push_str(" + &r#\"");
                    } else {
                        abort!(&f.ident, "nesting only work on inner structure")
                    }
                } else if nesting_format == Some(NestingFormat::Prefix) {
                    if let Some(field_type) = field_type {
                        field_example.push_str("\"#.to_string()");
                        if optional {
                            field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"\", \"# {field_name:}.\")"
                            ));
                        } else {
                            field_example.push_str(&format!(
                                " + &{field_type}::toml_field_example(\"\", \"{field_name:}.\")"
                            ));
                        }
                        field_example.push_str(" + &r#\"");
                    } else {
                        abort!(&f.ident, "nesting only work on inner structure")
                    }
                } else {
                    if optional {
                        field_example.push_str("# ");
                    }
                    match default {
                        DefaultSource::DefaultValue(default) => {
                            field_example.push_str("\"#.to_string() + prefix + &r#\"");
                            // TODO rename here
                            field_example.push_str(field_name.trim_start_matches("r#"));
                            field_example.push_str(" = ");
                            field_example.push_str(&default);
                            field_example.push('\n');
                        }
                        DefaultSource::DefaultFn(None) => {
                            field_example.push_str("\"#.to_string() + prefix + &r#\"");
                            field_example.push_str(&field_name);
                            field_example.push_str(" = \"\"\n");
                        }
                        DefaultSource::DefaultFn(Some(ty)) => {
                            field_example.push_str("\"#.to_string() + prefix + &r#\"");
                            field_example.push_str(&field_name);
                            field_example.push_str(" = \"#.to_string()");
                            field_example
                                .push_str(&format!(" + &format!(\"{{:?}}\",  {ty}::default())"));
                            field_example.push_str(" + &r#\"\n");
                        }
                        DefaultSource::SerdeDefaultFn(fn_str) => {
                            field_example.push_str("\"#.to_string() + prefix + &r#\"");
                            field_example.push_str(&field_name);
                            field_example.push_str(" = \"#.to_string()");
                            field_example
                                .push_str(&format!(" + &format!(\"{{:?}}\",  {fn_str}())"));
                            field_example.push_str("+ &r#\"\n");
                        }
                    }
                    field_example.push('\n');
                }
            }
        }
    }
    struct_doc.push_str("\"#.to_string()");
    field_example += &nesting_field_example;
    field_example.push_str("\"#.to_string()");

    let struct_doc_stream: proc_macro2::TokenStream =
        struct_doc.parse().expect("unexpected token in struct doc");
    let field_example_stream: proc_macro2::TokenStream =
        field_example.parse().expect("unexpected token in fields");

    let output = quote! {
        impl toml_example::TomlExample for #struct_name {
            fn toml_example() -> String {
                #struct_name::toml_field_example("", "")
            }
            fn toml_field_example(label: &str, prefix: &str) -> String {
                #struct_doc_stream + label + &#field_example_stream
            }
        }
    };
    TokenStream::from(output)
}
