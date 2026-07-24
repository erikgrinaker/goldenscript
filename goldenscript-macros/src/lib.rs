//! Derive macros for `goldenscript`.
//!
//! See the [command macro documentation](https://docs.rs/goldenscript/latest/goldenscript/#command-macro).

use std::collections::HashSet;

use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt as _;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, GenericArgument, Ident, LitStr, Meta,
    PathArguments, Type, Variant, parse_macro_input,
};

/// Specify helper attributes we "own". We'll ignore others.
const HELPER_ATTRIBUTES: [&str; 4] = ["command", "arg", "prefix", "tag"];

/// Derives command parsing via `TryFrom<&goldenscript::Command>`.
#[proc_macro_derive(Command, attributes(command, arg, prefix, tag))]
pub fn derive_command(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    expand_command(input).unwrap_or_else(Error::into_compile_error).into()
}

/// Expands `#[derive(Command)]` on an outer enum. Errors on non-enum types.
fn expand_command(input: DeriveInput) -> syn::Result<TokenStream> {
    // We can only expand non-generic enums.
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(&input.ident, "Command can only be derived for enums"));
    };
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "Command cannot be derived for generic enums",
        ));
    }

    // No helper attribute is valid here.
    if let Some(attr) = input.attrs.iter().find(|attr| is_helper_attr(attr)) {
        return Err(Error::new_spanned(attr, format!("{} is not valid on enum", quote!(#attr))));
    }

    // Expand the enum variants.
    let mut command_names = HashSet::new();
    let mut command_arms = Vec::new();
    let mut errors = None;
    for variant in &data.variants {
        match expand_variant(variant) {
            Ok((command_name, arm)) => {
                if !command_names.insert(command_name.clone()) {
                    combine_error(
                        &mut errors,
                        Error::new_spanned(
                            &variant.ident,
                            format!("duplicate command name '{command_name}'"),
                        ),
                    );
                    continue;
                }
                command_arms.push(arm);
            }
            Err(error) => combine_error(&mut errors, error),
        }
    }
    if let Some(error) = errors {
        return Err(error);
    }
    if command_arms.is_empty() {
        return Err(Error::new_spanned(
            &input.ident,
            "Command requires at least one command variant",
        ));
    }

    // Emit the TryFrom implementation.
    let ident = &input.ident;
    Ok(quote! {
        impl ::core::convert::TryFrom<&::goldenscript::Command> for #ident
        {
            type Error = ::std::boxed::Box<dyn ::std::error::Error>;

            fn try_from(
                command: &::goldenscript::Command,
            ) -> ::core::result::Result<Self, Self::Error> {
                match command.name.as_str() {
                    #(#command_arms)*
                    name => ::core::result::Result::Err(
                        ::std::format!("unknown command '{name}'").into()
                    ),
                }
            }
        }
    })
}

/// Expands an enum variant as a command. Returns the command name and match arm.
fn expand_variant(variant: &Variant) -> syn::Result<(String, TokenStream)> {
    let mut command_attr = None;
    let mut command_name = None;
    for attr in &variant.attrs {
        match helper_attr(attr) {
            Some("command") => {
                if command_attr.replace(attr).is_some() {
                    return Err(Error::new_spanned(attr, "duplicate #[command] attribute"));
                }
                parse_options(attr, |meta| {
                    if !meta.path.is_ident("name") {
                        return Err(meta.error("unknown command option"));
                    }
                    if command_name.is_some() {
                        return Err(meta.error("duplicate command name option"));
                    }
                    let value: LitStr = meta.value()?.parse()?;
                    command_name = Some(nonempty(value, "command name")?);
                    Ok(())
                })?;
            }
            Some(_) => return Err(Error::new_spanned(attr, "unexpected enum variant attribute")),
            None => continue,
        }
    }
    let command_name = command_name.unwrap_or_else(|| ident_name(&variant.ident).to_snake_case());

    let mut state = VariantState::default();
    let mut statements = Vec::with_capacity(variant.fields.len());
    let mut values = Vec::with_capacity(variant.fields.len());

    for (index, field) in variant.fields.iter().enumerate() {
        let field_name = field.ident.as_ref().map(ident_name);
        let label = field_name.clone().unwrap_or_else(|| format!("argument {}", index + 1));
        let binding = format_ident!("__field_{index}");
        let config = parse_field(field, field_name.as_deref())?;
        let parsed = expand_field(field, &config, &label, &mut state)?;
        let field_ty = &field.ty;
        statements.push(quote! {
            let #binding: #field_ty = #parsed;
        });
        values.push((field.ident.as_ref(), binding));
    }

    let unknown_tags = if state.has_tag_collection {
        quote! {}
    } else {
        let known = state.tag_names.iter();
        quote! {
            if let ::core::option::Option::Some(tag) = command
                .tags
                .iter()
                .find(|tag| ![#(#known),*].contains(&tag.as_str()))
            {
                return ::core::result::Result::Err(
                    ::std::format!("unexpected tag '{tag}'").into()
                );
            }
        }
    };

    let variant_ident = &variant.ident;
    let constructor = match &variant.fields {
        Fields::Unit => quote!(Self::#variant_ident),
        Fields::Unnamed(_) => {
            let bindings = values.iter().map(|(_, binding)| binding);
            quote!(Self::#variant_ident(#(#bindings),*))
        }
        Fields::Named(_) => {
            let fields = values.iter().map(|(ident, binding)| {
                let ident = ident.expect("named field");
                quote!(#ident: #binding)
            });
            quote!(Self::#variant_ident { #(#fields),* })
        }
    };

    let arm = quote! {
        #command_name => {
            let mut __args = command.consume_args();
            #(#statements)*
            __args.reject_next()?;
            #unknown_tags
            ::core::result::Result::Ok(#constructor)
        }
    };
    Ok((command_name, arm))
}

#[derive(Default)]
struct VariantState {
    positional: ArgumentState,
    keyed: ArgumentState,
    key_names: HashSet<String>,
    prefix_seen: bool,
    tag_names: Vec<String>,
    tag_name_set: HashSet<String>,
    has_tag_collection: bool,
}

#[derive(Default)]
struct ArgumentState {
    optional_seen: bool,
    variadic_seen: bool,
}

#[derive(Clone, Copy)]
enum ArgumentKind {
    Positional,
    Keyed,
}

enum FieldConfig {
    Argument { kind: Option<ArgumentKind>, key: Option<String>, default_key: Option<String> },
    Prefix,
    Tag { name: Option<String>, default_name: Option<String> },
}

fn expand_field(
    field: &Field,
    config: &FieldConfig,
    label: &str,
    state: &mut VariantState,
) -> syn::Result<TokenStream> {
    match config {
        FieldConfig::Argument { kind, key, default_key } => {
            expand_argument(field, *kind, key.as_deref(), default_key.as_deref(), label, state)
        }
        FieldConfig::Prefix => expand_prefix(field, label, state),
        FieldConfig::Tag { name, default_name } => {
            expand_tag(field, name.as_deref(), default_name.as_deref(), state)
        }
    }
}

fn expand_argument(
    field: &Field,
    configured_kind: Option<ArgumentKind>,
    key: Option<&str>,
    default_key: Option<&str>,
    label: &str,
    state: &mut VariantState,
) -> syn::Result<TokenStream> {
    let shape = classify_argument_type(&field.ty)?;
    let kind = match (&shape, configured_kind) {
        (TypeShape::Sequence(_, _), Some(ArgumentKind::Keyed)) => {
            return Err(Error::new_spanned(
                &field.ty,
                "sequence fields can only consume positional arguments",
            ));
        }
        (TypeShape::Sequence(_, _), _) => ArgumentKind::Positional,
        (TypeShape::Map(_), Some(ArgumentKind::Positional)) => {
            return Err(Error::new_spanned(
                &field.ty,
                "map fields can only consume key/value arguments",
            ));
        }
        (TypeShape::Map(_), _) => {
            if key.is_some() {
                return Err(Error::new_spanned(field, "map fields cannot have a key name"));
            }
            ArgumentKind::Keyed
        }
        (_, Some(kind)) => kind,
        (_, None) => ArgumentKind::Positional,
    };
    let key =
        if matches!(kind, ArgumentKind::Keyed) && !matches!(&shape, TypeShape::Map(_)) {
            Some(key.or(default_key).ok_or_else(|| {
                Error::new_spanned(field, "tuple key fields require key = \"...\"")
            })?)
        } else {
            None
        };

    let argument_state = match kind {
        ArgumentKind::Positional => &mut state.positional,
        ArgumentKind::Keyed => &mut state.keyed,
    };
    let (optional, variadic) = match shape {
        TypeShape::Scalar(_) => (false, false),
        TypeShape::Optional(_) => (true, false),
        TypeShape::Sequence(_, _) | TypeShape::Map(_) => (true, true),
    };
    if argument_state.variadic_seen {
        return Err(Error::new_spanned(
            field,
            "a variadic field must be the last field of its argument kind",
        ));
    }
    if !optional && argument_state.optional_seen {
        return Err(Error::new_spanned(
            field,
            "a required field cannot follow an optional field of the same argument kind",
        ));
    }
    argument_state.optional_seen |= optional;
    argument_state.variadic_seen |= variadic;

    if let Some(key) = key
        && !state.key_names.insert(key.to_owned())
    {
        return Err(Error::new_spanned(field, format!("duplicate argument key '{key}'")));
    }

    match shape {
        TypeShape::Scalar(ty) => {
            let source = required_argument_source(kind, key, label);
            Ok(parse_value(source, ty, label))
        }
        TypeShape::Optional(ty) => {
            let source = optional_argument_source(kind, key);
            let parsed = parse_value(quote!(__value), ty, label);
            Ok(quote! {
                match #source {
                    ::core::option::Option::Some(__value) => {
                        ::core::option::Option::Some(#parsed)
                    }
                    ::core::option::Option::None => ::core::option::Option::None,
                }
            })
        }
        TypeShape::Sequence(sequence, ty) => {
            let insert = match sequence {
                SequenceKind::Vec => quote!(__values.push(__parsed);),
                SequenceKind::HashSet | SequenceKind::BTreeSet => {
                    quote!(__values.insert(__parsed);)
                }
            };
            let field_ty = &field.ty;
            let parsed = parse_value(quote!(__value), ty, label);
            Ok(quote! {
                {
                    let mut __values: #field_ty = ::core::default::Default::default();
                    while let ::core::option::Option::Some(__value) = __args.next_pos() {
                        let __parsed = #parsed;
                        #insert
                    }
                    __values
                }
            })
        }
        TypeShape::Map(ty) => {
            let field_ty = &field.ty;
            let parsed = parse_value(quote!(__value), ty, label);
            Ok(quote! {
                {
                    let mut __values: #field_ty = ::core::default::Default::default();
                    while let ::core::option::Option::Some((__key, __value)) =
                        __args.next_key()
                    {
                        let __parsed = #parsed;
                        __values.insert(__key.to_owned(), __parsed);
                    }
                    __values
                }
            })
        }
    }
}

fn expand_prefix(field: &Field, label: &str, state: &mut VariantState) -> syn::Result<TokenStream> {
    if state.prefix_seen {
        return Err(Error::new_spanned(field, "only one prefix field is allowed"));
    }
    state.prefix_seen = true;

    match classify_argument_type(&field.ty)? {
        TypeShape::Scalar(ty) => {
            let source = quote! {
                command.prefix.as_deref().ok_or_else(
                    || -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("missing prefix for field '{}'", #label).into()
                    }
                )?
            };
            Ok(parse_value(source, ty, label))
        }
        TypeShape::Optional(ty) => {
            let parsed = parse_value(quote!(__value), ty, label);
            Ok(quote! {
                match command.prefix.as_deref() {
                    ::core::option::Option::Some(__value) => {
                        ::core::option::Option::Some(#parsed)
                    }
                    ::core::option::Option::None => ::core::option::Option::None,
                }
            })
        }
        TypeShape::Sequence(_, _) | TypeShape::Map(_) => {
            Err(Error::new_spanned(&field.ty, "prefix fields must be a scalar or Option<T>"))
        }
    }
}

fn expand_tag(
    field: &Field,
    name: Option<&str>,
    default_name: Option<&str>,
    state: &mut VariantState,
) -> syn::Result<TokenStream> {
    match classify_tag_type(&field.ty)? {
        TagShape::Bool => {
            let name = name.or(default_name).ok_or_else(|| {
                Error::new_spanned(field, "tuple boolean tag fields require #[tag(name = \"...\")]")
            })?;
            if !state.tag_name_set.insert(name.to_owned()) {
                return Err(Error::new_spanned(field, format!("duplicate tag name '{name}'")));
            }
            state.tag_names.push(name.to_owned());
            Ok(quote!(command.tags.contains(#name)))
        }
        TagShape::Collection => {
            if name.is_some() {
                return Err(Error::new_spanned(field, "tag collection fields cannot have a name"));
            }
            if state.has_tag_collection {
                return Err(Error::new_spanned(field, "only one tag collection field is allowed"));
            }
            state.has_tag_collection = true;
            let field_ty = &field.ty;
            Ok(quote! {
                command.tags.iter().cloned().collect::<#field_ty>()
            })
        }
    }
}

fn required_argument_source(kind: ArgumentKind, key: Option<&str>, label: &str) -> TokenStream {
    let value = optional_argument_source(kind, key);
    quote! {
        #value.ok_or_else(
            || -> ::std::boxed::Box<dyn ::std::error::Error> {
                ::std::format!("missing argument for field '{}'", #label).into()
            }
        )?
    }
}

fn optional_argument_source(kind: ArgumentKind, key: Option<&str>) -> TokenStream {
    match kind {
        ArgumentKind::Positional => quote!(__args.next_pos()),
        ArgumentKind::Keyed => {
            let key = key.expect("keyed argument has a key");
            quote!(__args.take_key(#key))
        }
    }
}

fn parse_value(source: TokenStream, ty: &Type, label: &str) -> TokenStream {
    quote! {
        {
            let __value = #source;
            __value.parse::<#ty>().map_err(
                |_| -> ::std::boxed::Box<dyn ::std::error::Error> {
                    ::std::format!("invalid value '{__value}' for field '{}'", #label).into()
                }
            )?
        }
    }
}

fn parse_field(field: &Field, field_name: Option<&str>) -> syn::Result<FieldConfig> {
    if field.attrs.iter().any(|attr| attr.path().is_ident("command")) {
        return Err(Error::new_spanned(field, "#[command] is only valid on enum variants"));
    }

    let role_attrs: Vec<_> = field
        .attrs
        .iter()
        .filter(|attr| {
            attr.path().is_ident("arg")
                || attr.path().is_ident("prefix")
                || attr.path().is_ident("tag")
        })
        .collect();
    if role_attrs.len() > 1 {
        return Err(Error::new_spanned(
            role_attrs[1],
            "a field can only have one of #[arg], #[prefix], or #[tag]",
        ));
    }
    let Some(attr) = role_attrs.first() else {
        return Ok(FieldConfig::Argument {
            kind: None,
            key: None,
            default_key: field_name.map(str::to_owned),
        });
    };

    if attr.path().is_ident("prefix") {
        ensure_bare(attr, "#[prefix] does not accept options")?;
        return Ok(FieldConfig::Prefix);
    }
    if attr.path().is_ident("tag") {
        let mut name = None;
        parse_options(attr, |meta| {
            if !meta.path.is_ident("name") {
                return Err(meta.error("unknown tag option"));
            }
            if name.is_some() {
                return Err(meta.error("duplicate tag name option"));
            }
            let value: LitStr = meta.value()?.parse()?;
            name = Some(nonempty(value, "tag name")?);
            Ok(())
        })?;
        return Ok(FieldConfig::Tag { name, default_name: field_name.map(str::to_owned) });
    }

    let mut positional = false;
    let mut key = None;
    let mut key_seen = false;
    parse_options(attr, |meta| {
        if meta.path.is_ident("pos") {
            if positional {
                return Err(meta.error("duplicate pos option"));
            }
            if meta.input.peek(syn::Token![=]) || meta.input.peek(syn::token::Paren) {
                return Err(meta.error("pos does not accept a value"));
            }
            positional = true;
            return Ok(());
        }
        if meta.path.is_ident("key") {
            if key_seen {
                return Err(meta.error("duplicate key option"));
            }
            key_seen = true;
            if meta.input.peek(syn::Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                key = Some(nonempty(value, "argument key")?);
            }
            return Ok(());
        }
        Err(meta.error("unknown argument option"))
    })?;
    if positional && key_seen {
        return Err(Error::new_spanned(attr, "pos and key cannot be used together"));
    }
    Ok(FieldConfig::Argument {
        kind: if key_seen {
            Some(ArgumentKind::Keyed)
        } else if positional {
            Some(ArgumentKind::Positional)
        } else {
            None
        },
        key,
        default_key: field_name.map(str::to_owned),
    })
}

/// Returns the name of the given attribute if it is one of our helper attributes.
fn helper_attr(attr: &Attribute) -> Option<&str> {
    let ident = attr.path().get_ident()?;
    HELPER_ATTRIBUTES.into_iter().find(|name| ident == name)
}

fn is_helper_attr(attr: &Attribute) -> bool {
    helper_attr(attr).is_some()
}

fn parse_options(
    attr: &Attribute,
    logic: impl FnMut(syn::meta::ParseNestedMeta<'_>) -> syn::Result<()>,
) -> syn::Result<()> {
    match &attr.meta {
        Meta::Path(_) => Ok(()),
        Meta::List(_) => attr.parse_nested_meta(logic),
        Meta::NameValue(_) => Err(Error::new_spanned(attr, "expected attribute options")),
    }
}

fn ensure_bare(attr: &Attribute, message: &str) -> syn::Result<()> {
    if matches!(attr.meta, Meta::Path(_)) { Ok(()) } else { Err(Error::new_spanned(attr, message)) }
}

fn nonempty(value: LitStr, description: &str) -> syn::Result<String> {
    let value_string = value.value();
    if value_string.is_empty() {
        Err(Error::new(value.span(), format!("{description} cannot be empty")))
    } else {
        Ok(value_string)
    }
}

enum TypeShape<'a> {
    Scalar(&'a Type),
    Optional(&'a Type),
    Sequence(SequenceKind, &'a Type),
    Map(&'a Type),
}

#[derive(Clone, Copy)]
enum SequenceKind {
    Vec,
    HashSet,
    BTreeSet,
}

enum TagShape {
    Bool,
    Collection,
}

fn classify_argument_type(ty: &Type) -> syn::Result<TypeShape<'_>> {
    let Some((ident, arguments)) = type_path_parts(ty) else {
        return Ok(TypeShape::Scalar(ty));
    };
    match ident.to_string().as_str() {
        "Option" => {
            let inner = one_type_argument(arguments, ty, "Option")?;
            reject_nested_container(inner)?;
            Ok(TypeShape::Optional(inner))
        }
        "Vec" => {
            let inner = one_type_argument(arguments, ty, "Vec")?;
            reject_nested_container(inner)?;
            Ok(TypeShape::Sequence(SequenceKind::Vec, inner))
        }
        "HashSet" => {
            let inner = one_type_argument(arguments, ty, "HashSet")?;
            reject_nested_container(inner)?;
            Ok(TypeShape::Sequence(SequenceKind::HashSet, inner))
        }
        "BTreeSet" => {
            let inner = one_type_argument(arguments, ty, "BTreeSet")?;
            reject_nested_container(inner)?;
            Ok(TypeShape::Sequence(SequenceKind::BTreeSet, inner))
        }
        "HashMap" | "BTreeMap" => map_type(arguments, ty),
        _ => Ok(TypeShape::Scalar(ty)),
    }
}

fn map_type<'a>(arguments: &'a PathArguments, ty: &'a Type) -> syn::Result<TypeShape<'a>> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return Err(Error::new_spanned(ty, "map type requires String and value types"));
    };
    let types: Vec<_> = arguments
        .args
        .iter()
        .filter_map(|argument| match argument {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect();
    if types.len() != 2 || arguments.args.len() != 2 {
        return Err(Error::new_spanned(ty, "map type requires String and value types"));
    }
    if !is_named_type(types[0], "String") {
        return Err(Error::new_spanned(types[0], "map argument keys must be String"));
    }
    reject_nested_container(types[1])?;
    Ok(TypeShape::Map(types[1]))
}

fn classify_tag_type(ty: &Type) -> syn::Result<TagShape> {
    if is_named_type(ty, "bool") {
        return Ok(TagShape::Bool);
    }
    let Some((ident, arguments)) = type_path_parts(ty) else {
        return Err(Error::new_spanned(
            ty,
            "tag fields must be bool or a supported String collection",
        ));
    };
    if !matches!(ident.to_string().as_str(), "Vec" | "HashSet" | "BTreeSet") {
        return Err(Error::new_spanned(
            ty,
            "tag fields must be bool or a supported String collection",
        ));
    }
    let inner = one_type_argument(arguments, ty, "tag collection")?;
    if !is_named_type(inner, "String") {
        return Err(Error::new_spanned(inner, "tag collections must contain String"));
    }
    Ok(TagShape::Collection)
}

fn type_path_parts(ty: &Type) -> Option<(&Ident, &PathArguments)> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    Some((&segment.ident, &segment.arguments))
}

fn one_type_argument<'a>(
    arguments: &'a PathArguments,
    ty: &'a Type,
    name: &str,
) -> syn::Result<&'a Type> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return Err(Error::new_spanned(ty, format!("{name} requires one type argument")));
    };
    if arguments.args.len() != 1 {
        return Err(Error::new_spanned(ty, format!("{name} requires one type argument")));
    }
    match arguments.args.first() {
        Some(GenericArgument::Type(ty)) => Ok(ty),
        _ => Err(Error::new_spanned(ty, format!("{name} requires one type argument"))),
    }
}

fn reject_nested_container(ty: &Type) -> syn::Result<()> {
    let Some((ident, _)) = type_path_parts(ty) else {
        return Ok(());
    };
    if matches!(
        ident.to_string().as_str(),
        "Option" | "Vec" | "HashSet" | "BTreeSet" | "HashMap" | "BTreeMap"
    ) {
        Err(Error::new_spanned(ty, "nested option and collection types are not supported"))
    } else {
        Ok(())
    }
}

fn is_named_type(ty: &Type, name: &str) -> bool {
    let Some((ident, arguments)) = type_path_parts(ty) else {
        return false;
    };
    ident == name && matches!(arguments, PathArguments::None)
}

fn ident_name(ident: &Ident) -> String {
    ident.unraw().to_string()
}

fn combine_error(errors: &mut Option<Error>, error: Error) {
    if let Some(errors) = errors {
        errors.combine(error);
    } else {
        *errors = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_command_name() -> syn::Result<()> {
        let input: DeriveInput = syn::parse_quote! {
            enum Commands {
                HTTPRequest,
                #[command(name = "renamed")]
                Other,
            }
        };
        let Data::Enum(data) = input.data else { unreachable!() };
        let (command_name, _) = expand_variant(&data.variants[0])?;
        assert_eq!(command_name, "http_request");
        let (command_name, _) = expand_variant(&data.variants[1])?;
        assert_eq!(command_name, "renamed");
        Ok(())
    }

    #[test]
    fn classifies_argument_types() -> syn::Result<()> {
        let scalar: Type = syn::parse_quote!(u64);
        let optional: Type = syn::parse_quote!(Option<u64>);
        let sequence: Type = syn::parse_quote!(std::collections::BTreeSet<u64>);
        let map: Type = syn::parse_quote!(std::collections::HashMap<String, u64>);
        assert!(matches!(classify_argument_type(&scalar)?, TypeShape::Scalar(_)));
        assert!(matches!(classify_argument_type(&optional)?, TypeShape::Optional(_)));
        assert!(matches!(
            classify_argument_type(&sequence)?,
            TypeShape::Sequence(SequenceKind::BTreeSet, _)
        ));
        assert!(matches!(classify_argument_type(&map)?, TypeShape::Map(_)));
        Ok(())
    }

    #[test]
    fn parses_field_attributes() -> syn::Result<()> {
        let input: DeriveInput = syn::parse_quote! {
            enum Commands {
                Command {
                    positional: String,
                    #[arg(key)]
                    keyed: u64,
                    #[prefix]
                    prefix: Option<char>,
                    #[tag(name = "verbose")]
                    debug: bool,
                }
            }
        };
        let Data::Enum(data) = input.data else { unreachable!() };
        let fields: Vec<_> = data.variants[0].fields.iter().collect();
        assert!(matches!(
            parse_field(fields[0], Some("positional"))?,
            FieldConfig::Argument { kind: None, .. }
        ));
        assert!(matches!(
            parse_field(fields[1], Some("keyed"))?,
            FieldConfig::Argument {
                kind: Some(ArgumentKind::Keyed),
                default_key: Some(key),
                ..
            } if key == "keyed"
        ));
        assert!(matches!(parse_field(fields[2], Some("prefix"))?, FieldConfig::Prefix));
        assert!(matches!(
            parse_field(fields[3], Some("debug"))?,
            FieldConfig::Tag {
                name: Some(name),
                ..
            } if name == "verbose"
        ));
        Ok(())
    }

    #[test]
    fn combines_variant_errors() -> syn::Result<()> {
        let input: DeriveInput = syn::parse_quote! {
            enum Commands {
                #[arg]
                One,
                #[prefix]
                Two,
                #[tag]
                Three,
            }
        };
        let error = expand_command(input).unwrap_err().into_compile_error().to_string();
        assert_eq!(error.matches("only #[command] is valid on enum variants").count(), 3);
        Ok(())
    }
}
