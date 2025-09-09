extern crate proc_macro;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error2::OptionExt;
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

struct AttrMeta {
    docs: Vec<String>,
    default_source: Option<DefaultSource>,
    nesting_format: Option<NestingFormat>,
    require: bool,
    skip: bool,
    is_enum: bool,
    flatten: bool,
    rename: Option<String>,
    rename_rule: case::RenameRule,
}

struct ParsedField {
    docs: Vec<String>,
    default: DefaultSource,
    nesting_format: Option<NestingFormat>,
    skip: bool,
    is_enum: bool,
    flatten: bool,
    name: String,
    optional: bool,
    ty: Option<String>,
}

impl ParsedField {
    fn push_doc_to_string(&self, s: &mut String) {
        push_doc_string(s, &self.docs);
    }

    // Provide a default key for map-like example
    fn default_key(&self) -> String {
        if let DefaultSource::DefaultValue(v) = &self.default {
            let key = v.trim_matches('\"').replace(' ', "").replace('.', "-");
            if !key.is_empty() {
                return key;
            }
        }
        "example".into()
    }

    fn label(&self) -> String {
        match self.nesting_format {
            Some(NestingFormat::Section(NestingType::Vec)) => {
                if self.flatten {
                    abort!(
                        "flatten",
                        format!(
                            "Only structs and maps can be flattened! \
                            (But field `{}` is a collection)",
                            self.name
                        )
                    )
                }
                self.prefix() + &format!("[[{}]]", self.name)
            }
            Some(NestingFormat::Section(NestingType::Dict)) => {
                self.prefix()
                    + &if self.flatten {
                        format!("[{}]", self.default_key())
                    } else {
                        format!("[{}.{}]", self.name, self.default_key())
                    }
            }
            Some(NestingFormat::Prefix) => "".to_string(),
            _ => {
                if self.flatten {
                    self.prefix()
                } else {
                    self.prefix() + &format!("[{}]", self.name)
                }
            }
        }
    }

    fn prefix(&self) -> String {
        let opt_prefix = if self.optional {
            "# ".to_string()
        } else {
            String::new()
        };
        if self.nesting_format == Some(NestingFormat::Prefix) {
            format!("{}{}.", opt_prefix, self.name)
        } else {
            opt_prefix
        }
    }
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

fn parse_attrs(attrs: &[Attribute]) -> AttrMeta {
    let mut docs = Vec::new();
    let mut default_source = None;
    let mut nesting_format = None;
    let mut require = false;
    let mut skip = false;
    let mut is_enum = false;
    let mut flatten = false;
    // mut in serde feature
    #[allow(unused_mut)]
    let mut rename = None;
    // mut in serde feature
    #[allow(unused_mut)]
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
            ) if path.segments.last().is_some_and(|s| s.ident == "serde") => {
                #[cfg(feature = "serde")]
                {
                    let token_str = _tokens.to_string();
                    for attribute in token_str.split(find_unenclosed_char(',')).map(str::trim) {
                        if attribute.starts_with("default") {
                            if let Some((_, s)) = attribute.split_once('=') {
                                default_source = Some(DefaultSource::SerdeDefaultFn(
                                    s.trim().trim_matches('"').into(),
                                ));
                            } else {
                                default_source = Some(DefaultSource::DefaultFn(None));
                            }
                        }
                        if attribute == "skip_deserializing" || attribute == "skip" {
                            skip = true;
                        }
                        if attribute == "flatten" {
                            flatten = true;
                        }
                        if attribute.starts_with("rename") {
                            if attribute.starts_with("rename_all") {
                                if let Some((_, s)) = attribute.split_once('=') {
                                    rename_rule = if let Ok(r) =
                                        case::RenameRule::from_str(s.trim().trim_matches('"'))
                                    {
                                        r
                                    } else {
                                        abort!(&_tokens, "unsupported rename rule")
                                    }
                                }
                            } else if let Some((_, s)) = attribute.split_once('=') {
                                rename = Some(s.trim().trim_matches('"').into());
                            }
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
                for attribute in token_str.split(find_unenclosed_char(',')).map(str::trim) {
                    if attribute.starts_with("default") {
                        if let Some((_, s)) = attribute.split_once('=') {
                            default_source = Some(DefaultSource::DefaultValue(s.trim().into()));
                        } else {
                            default_source = Some(DefaultSource::DefaultFn(None));
                        }
                    } else if attribute.starts_with("nesting") {
                        if let Some((_, s)) = attribute.split_once('=') {
                            nesting_format = match s.trim() {
                                "prefix" => Some(NestingFormat::Prefix),
                                "section" => Some(NestingFormat::Section(NestingType::None)),
                                _ => {
                                    abort!(&attr, "please use prefix or section for nesting derive")
                                }
                            }
                        } else {
                            nesting_format = Some(NestingFormat::Section(NestingType::None));
                        }
                    } else if attribute == "require" {
                        require = true;
                    } else if attribute == "skip" {
                        skip = true;
                    } else if attribute == "is_enum" || attribute == "enum" {
                        is_enum = true;
                    } else if attribute == "flatten" {
                        flatten = true;
                    } else {
                        abort!(&attr, format!("{} is not allowed attribute", attribute))
                    }
                }
            }
            _ => (),
        }
    }

    AttrMeta {
        docs,
        default_source,
        nesting_format,
        require,
        skip,
        is_enum,
        flatten,
        rename,
        rename_rule,
    }
}

fn parse_field(
    struct_default: Option<&DefaultSource>,
    field: &Field,
    rename_rule: case::RenameRule,
) -> ParsedField {
    let mut default_value = String::new();
    let mut optional = false;
    let AttrMeta {
        docs,
        default_source,
        mut nesting_format,
        skip,
        is_enum,
        flatten,
        rename,
        require,
        ..
    } = parse_attrs(&field.attrs);
    let ty = parse_type(
        &field.ty,
        &mut default_value,
        &mut optional,
        &mut nesting_format,
    );
    let default = match default_source {
        Some(DefaultSource::DefaultFn(_)) => DefaultSource::DefaultFn(ty.clone()),
        Some(DefaultSource::SerdeDefaultFn(f)) => DefaultSource::SerdeDefaultFn(f),
        Some(DefaultSource::DefaultValue(v)) => DefaultSource::DefaultValue(v),
        _ if struct_default.is_some() => DefaultSource::DefaultFn(None),
        _ => DefaultSource::DefaultValue(default_value),
    };
    let name = if let Some(field_name) = field.ident.as_ref().map(|i| i.to_string()) {
        rename.unwrap_or(rename_rule.apply_to_field(&field_name))
    } else {
        abort!(&field, "The field should has name")
    };
    ParsedField {
        docs,
        default,
        nesting_format,
        skip,
        is_enum,
        flatten,
        name,
        optional: optional && !require,
        ty,
    }
}

fn push_doc_string(example: &mut String, docs: &[String]) {
    for doc in docs.iter() {
        example.push('#');
        example.push_str(doc);
        example.push('\n');
    }
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

        let AttrMeta {
            docs,
            default_source,
            rename_rule,
            ..
        } = parse_attrs(&attrs);

        let struct_doc = {
            let mut doc = String::new();
            push_doc_string(&mut doc, &docs);
            doc
        };

        let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &data {
            fields
        } else {
            abort!(ident, "TomlExample derive only use for struct")
        };

        let field_example = Self::parse_field_examples(ident, default_source, fields, rename_rule);

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
                fn toml_example_with_prefix(label: &str, prefix: &str) -> String {
                    #struct_doc.to_string() + label + &#field_example_stream
                }
            }
        })
    }

    fn parse_field_examples(
        struct_ty: Ident,
        struct_default: Option<DefaultSource>,
        fields: &Fields,
        rename_rule: case::RenameRule,
    ) -> String {
        let mut field_example = "r##\"".to_string();
        let mut nesting_field_example = "".to_string();

        if let Named(named_fields) = fields {
            for f in named_fields.named.iter() {
                let field = parse_field(struct_default.as_ref(), f, rename_rule);
                if field.skip {
                    continue;
                }

                if field.nesting_format.is_some() {
                    // Recursively add the toml_example_with_prefix of fields
                    // If nesting in a section way will attached to the bottom to avoid #18
                    // else the nesting will just using a prefix ahead the every field of example
                    let (example, nesting_section_newline) =
                        if field.nesting_format == Some(NestingFormat::Prefix) {
                            (&mut field_example, "")
                        } else {
                            (
                                &mut nesting_field_example,
                                if field.flatten { "" } else { "\n" },
                            )
                        };

                    field.push_doc_to_string(example);
                    if let Some(ref field_type) = field.ty {
                        example.push_str("\"##.to_string()");
                        example.push_str(&format!(
                            " + &{field_type}::toml_example_with_prefix(\"{}{}\", \"{}\")",
                            field.label(),
                            nesting_section_newline,
                            field.prefix()
                        ));
                        example.push_str(" + &r##\"");
                    } else {
                        abort!(&f.ident, "nesting only work on inner structure")
                    }
                } else {
                    // The leaf field, writing down the example value based on different default source
                    field.push_doc_to_string(&mut field_example);
                    if field.optional {
                        field_example.push_str("# ");
                    }
                    field_example.push_str("\"##.to_string() + prefix + &r##\"");
                    field_example.push_str(field.name.trim_start_matches("r#"));
                    match field.default {
                        DefaultSource::DefaultValue(default) => {
                            field_example.push_str(" = ");
                            field_example.push_str(&default);
                            field_example.push('\n');
                        }
                        DefaultSource::DefaultFn(None) => match struct_default {
                            Some(DefaultSource::DefaultFn(None)) => {
                                let suffix = format!(
                                    ".{}",
                                    f.ident
                                        .as_ref()
                                        .expect_or_abort("Named fields always have and ident")
                                );
                                handle_default_fn_source(
                                    &mut field_example,
                                    field.is_enum,
                                    struct_ty.to_string(),
                                    Some(suffix),
                                );
                            }
                            Some(DefaultSource::SerdeDefaultFn(ref fn_str)) => {
                                let suffix = format!(
                                    ".{}",
                                    f.ident
                                        .as_ref()
                                        .expect_or_abort("Named fields always have an ident")
                                );
                                handle_serde_default_fn_source(
                                    &mut field_example,
                                    field.is_enum,
                                    fn_str,
                                    Some(suffix),
                                );
                            }
                            Some(DefaultSource::DefaultValue(_)) => abort!(
                                f.ident,
                                "Setting a default value on a struct is not supported!"
                            ),
                            _ => field_example.push_str(" = \"\"\n"),
                        },
                        DefaultSource::DefaultFn(Some(ty)) => {
                            handle_default_fn_source(&mut field_example, field.is_enum, ty, None)
                        }
                        DefaultSource::SerdeDefaultFn(ref fn_str) => {
                            handle_serde_default_fn_source(
                                &mut field_example,
                                field.is_enum,
                                fn_str,
                                None,
                            )
                        }
                    }
                    field_example.push('\n');
                }
            }
        }
        field_example += &nesting_field_example;
        field_example.push_str("\"##.to_string()");

        field_example
    }
}

fn handle_default_fn_source(
    field_example: &mut String,
    is_enum: bool,
    type_ident: String,
    suffix: Option<String>,
) {
    let suffix = suffix.unwrap_or_default();
    field_example.push_str(" = \"##.to_string()");
    if is_enum {
        field_example.push_str(&format!(
            " + &format!(\"\\\"{{:?}}\\\"\",  {type_ident}::default(){suffix})"
        ));
    } else {
        field_example.push_str(&format!(
            " + &format!(\"{{:?}}\",  {type_ident}::default(){suffix})"
        ));
    }
    field_example.push_str(" + &r##\"\n");
}

fn handle_serde_default_fn_source(
    field_example: &mut String,
    is_enum: bool,
    fn_str: &String,
    suffix: Option<String>,
) {
    let suffix = suffix.unwrap_or_default();
    field_example.push_str(" = \"##.to_string()");
    if is_enum {
        field_example.push_str(&format!(
            " + &format!(\"\\\"{{:?}}\\\"\",  {fn_str}(){suffix})"
        ));
    } else {
        field_example.push_str(&format!(" + &format!(\"{{:?}}\",  {fn_str}(){suffix})"));
    }
    field_example.push_str("+ &r##\"\n");
}

/// A [Pattern](std::str::pattern::Pattern) to find a char that is not enclosed in quotes, braces
/// or the like
fn find_unenclosed_char(pat: char) -> impl FnMut(char) -> bool {
    let mut quotes = 0;
    let mut single_quotes = 0;
    let mut brackets = 0;
    let mut braces = 0;
    let mut parenthesis = 0;
    let mut is_escaped = false;
    move |char| -> bool {
        if is_escaped {
            is_escaped = false;
            return false;
        } else if char == '\\' {
            is_escaped = true;
        } else if (quotes % 2 == 1 && char != '"') || (single_quotes % 2 == 1 && char != '\'') {
            return false;
        } else {
            match char {
                '"' => quotes += 1,
                '\'' => single_quotes += 1,
                '[' => brackets += 1,
                ']' => brackets -= 1,
                '{' => braces += 1,
                '}' => braces -= 1,
                '(' => parenthesis += 1,
                ')' => parenthesis -= 1,
                _ => {}
            }
        }
        char == pat
            && quotes % 2 == 0
            && single_quotes % 2 == 0
            && brackets == 0
            && braces == 0
            && parenthesis == 0
    }
}
