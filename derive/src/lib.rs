extern crate proc_macro;

use proc_macro2::TokenStream;
use proc_macro2::Ident;
use proc_macro_error2::{abort, proc_macro_error};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments,
    AttrStyle::Outer,
    Attribute, DeriveInput,
    Expr::Lit,
    ExprLit, Field, Fields,
    Fields::Named,
    GenericArgument,
    Lit::Str,
    Meta::{List, NameValue},
    MetaList, MetaNameValue, PathArguments, PathSegment, Result, Type, TypePath,
};
mod case;

struct Intermediate {
    struct_name: Ident,
    struct_doc: String,
    field_example: String,
}

struct FieldMeta {
    docs: Vec<String>,
    default_source: Option<DefaultSource>,
    nesting_format: Option<NestingFormat>,
    require: bool,
    skip: bool,
    rename: Option<String>,
    rename_rule: case::RenameRule,
}

struct ParsedField {
    docs: Vec<String>,
    default: DefaultSource,
    nesting_format: Option<NestingFormat>,
    skip: bool,
    rename: Option<String>,
    optional: bool,
}

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

/// return type and unwrap with Option and Vec; or return the value type of HashMap and BTreeMap
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

fn parse_attrs(
    attrs: &[Attribute],
) -> FieldMeta {
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

    FieldMeta{
        docs,
        default_source,
        nesting_format,
        require,
        skip,
        rename,
        rename_rule,
    }
}

fn parse_field(
    field: &Field,
) -> ParsedField {
    let mut default_value = String::new();
    let mut optional = false;
    let FieldMeta {docs, default_source, mut nesting_format, skip, rename, require, .. } =
        parse_attrs(&field.attrs);
    let ty = parse_type(
        &field.ty,
        &mut default_value,
        &mut optional,
        &mut nesting_format,
    );
    let default = match default_source {
        Some(DefaultSource::DefaultFn(_)) => DefaultSource::DefaultFn(ty),
        Some(DefaultSource::SerdeDefaultFn(f)) => DefaultSource::SerdeDefaultFn(f),
        Some(DefaultSource::DefaultValue(v)) => DefaultSource::DefaultValue(v),
        _ => DefaultSource::DefaultValue(default_value),
    };
    ParsedField {
        docs,
        default,
        nesting_format,
        skip,
        rename,
        optional:  optional && !require,
    }
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
pub fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Intermediate::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
}

// Transient intermediate for structure parsing
impl Intermediate {
    pub fn from_ast(
        DeriveInput {
            ident, data, attrs, ..
        }: syn::DeriveInput,
    ) -> Result<Intermediate> {
        let struct_name = ident.clone();

        let FieldMeta{ docs, rename_rule, .. } = parse_attrs(&attrs);

        let struct_doc = {
            let mut doc = String::new();
            push_doc_string(&mut doc, docs);
            doc
        };

        let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &data {
            fields
        } else {
            abort!(ident, "TomlExample derive only use for struct")
        };

        let field_example = Self::parse_field_examples(fields, rename_rule);

        Ok(Intermediate {
            struct_name,
            struct_doc,
            field_example,
        })
    }

    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Intermediate {
            struct_name,
            struct_doc,
            field_example,
        } = self;

        let field_example_stream: proc_macro2::TokenStream = field_example.parse()?;

        Ok(quote! {
            impl toml_example::TomlExample for #struct_name {
                fn toml_example() -> String {
                    #struct_name::toml_example_with_prefix("", "")
                }
                fn toml_example_with_prefix(label: &str, prefix: &str) -> String{
                    #struct_doc.to_string() + label + &#field_example_stream
                }
            }
        })
    }

    fn parse_field_examples(fields: &Fields, rename_rule: case::RenameRule) -> String {
        // Always put nesting field example in the last to avoid #18
        let mut field_example = "r##\"".to_string();
        let mut nesting_field_example = "".to_string();

        if let Named(named_fields) = fields {
            for f in named_fields.named.iter() {
                let field_type = parse_type(&f.ty, &mut String::new(), &mut false, &mut None);
                if let Some(mut field_name) = f.ident.as_ref().map(|i| i.to_string()) {
                    let field = parse_field(f);
                    if field.skip {
                        continue;
                    }
                    if let Some(name) = field.rename {
                        field_name = name;
                    } else {
                        field_name = rename_rule.apply_to_field(&field_name);
                    }
                    if field.nesting_format
                        .as_ref()
                        .map(|f| matches!(f, NestingFormat::Section(_)))
                        .unwrap_or_default()
                    {
                        if let Some(field_type) = field_type {
                            push_doc_string(&mut nesting_field_example, field.docs);
                            nesting_field_example.push_str("\"##.to_string()");
                            let key = default_key(field.default);
                            match field.nesting_format {
                                Some(NestingFormat::Section(NestingType::Vec)) if field.optional => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"# [[{field_name:}]]\n\", \"# \")"
                                )),
                                Some(NestingFormat::Section(NestingType::Vec)) => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"[[{field_name:}]]\n\", \"\")"
                                )),
                                Some(NestingFormat::Section(NestingType::Dict)) if field.optional => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"# [{field_name:}.{key}]\n\", \"# \")"
                                )),
                                Some(NestingFormat::Section(NestingType::Dict)) => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"[{field_name:}.{key}]\n\", \"\")"
                                )),
                                _ if field.optional => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"# [{field_name:}]\n\", \"# \")"
                                )),
                                _ => nesting_field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"[{field_name:}]\n\", \"\")"
                                ))
                            };
                            nesting_field_example.push_str(" + &r##\"");
                        } else {
                            abort!(&f.ident, "nesting only work on inner structure")
                        }
                    } else if field.nesting_format == Some(NestingFormat::Prefix) {
                        push_doc_string(&mut field_example, field.docs);
                        if let Some(field_type) = field_type {
                            field_example.push_str("\"##.to_string()");
                            if field.optional {
                                field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"\", \"# {field_name:}.\")"
                                ));
                            } else {
                                field_example.push_str(&format!(
                                    " + &{field_type}::toml_example_with_prefix(\"\", \"{field_name:}.\")"
                                ));
                            }
                            field_example.push_str(" + &r##\"");
                        } else {
                            abort!(&f.ident, "nesting only work on inner structure")
                        }
                    } else {
                        push_doc_string(&mut field_example, field.docs);
                        if field.optional {
                            field_example.push_str("# ");
                        }
                        match field.default {
                            DefaultSource::DefaultValue(default) => {
                                field_example.push_str("\"##.to_string() + prefix + &r##\"");
                                field_example.push_str(field_name.trim_start_matches("r#"));
                                field_example.push_str(" = ");
                                field_example.push_str(&default);
                                field_example.push('\n');
                            }
                            DefaultSource::DefaultFn(None) => {
                                field_example.push_str("\"##.to_string() + prefix + &r##\"");
                                field_example.push_str(&field_name);
                                field_example.push_str(" = \"\"\n");
                            }
                            DefaultSource::DefaultFn(Some(ty)) => {
                                field_example.push_str("\"##.to_string() + prefix + &r##\"");
                                field_example.push_str(&field_name);
                                field_example.push_str(" = \"##.to_string()");
                                field_example
                                    .push_str(&format!(" + &format!(\"{{:?}}\",  {ty}::default())"));
                                field_example.push_str(" + &r##\"\n");
                            }
                            DefaultSource::SerdeDefaultFn(fn_str) => {
                                field_example.push_str("\"##.to_string() + prefix + &r##\"");
                                field_example.push_str(&field_name);
                                field_example.push_str(" = \"##.to_string()");
                                field_example.push_str(&format!(
                                    " + &format!(\"{{:?}}\",  {fn_str}())"
                                ));
                                field_example.push_str("+ &r##\"\n");
                            }
                        }
                        field_example.push('\n');
                    }
                }
            }
        }
        field_example += &nesting_field_example;
        field_example.push_str("\"##.to_string()");

        field_example
    }
}
