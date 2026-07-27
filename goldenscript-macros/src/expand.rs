use std::collections::HashSet;

use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, GenericArgument, Ident, LitStr, Meta,
    PathArguments, Result, Type, Variant, meta::ParseNestedMeta, parse::Parse,
};

/// Specify helper attributes we "own". We handle these and ignore other attributes.
const HELPER_ATTRIBUTES: [&str; 2] = ["command", "arg"];

fn is_helper_attr(attr: &Attribute) -> bool {
    let Some(ident) = attr.path().get_ident() else {
        return false;
    };
    HELPER_ATTRIBUTES.into_iter().find(|name| ident == name).is_some()
}

/// Expands `#[derive(Command)]`.
#[derive(Default)]
pub struct CommandExpander {
    command_names: HashSet<String>,
    has_other: bool,
}

impl CommandExpander {
    /// Creates a new CommandExpander.
    pub fn new() -> Self {
        Self::default()
    }

    /// Expands `#[derive(Command)]` on an outer enum. Errors on non-enum types.
    pub fn expand(mut self, input: DeriveInput) -> Result<TokenStream> {
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
            return Err(Error::new_spanned(
                attr,
                format!("{} is not valid on enum", quote!(#attr)),
            ));
        }

        // Expand the enum variants.
        if data.variants.is_empty() {
            return Err(Error::new_spanned(
                &input.ident,
                "Command requires at least one command variant",
            ));
        }
        let command_arms = data
            .variants
            .iter()
            .map(|variant| VariantExpander::new(&mut self).expand(variant))
            .collect::<Result<Vec<_>>>()?;

        // Emit the TryFrom implementation.
        let ident = &input.ident;
        let unknown_arm = (!self.has_other).then(|| {
            quote! {
                name => ::core::result::Result::Err(
                    ::std::format!("unknown command '{name}'").into()
                ),
            }
        });

        Ok(quote! {
            impl ::core::convert::TryFrom<&::goldenscript::Command> for #ident
            {
                type Error = ::std::boxed::Box<dyn ::std::error::Error>;

                fn try_from(
                    command: &::goldenscript::Command,
                ) -> ::core::result::Result<
                    Self,
                    ::std::boxed::Box<dyn ::std::error::Error>,
                > {
                    match command.name.as_str() {
                        #(#command_arms)*
                        #unknown_arm
                    }
                }
            }
        })
    }
}

/// Expands an enum variant as a command. Holds a reference to the `CommandExpander` for
/// cross-variant state, e.g. to avoid duplicate command names.
pub struct VariantExpander<'a> {
    command_expander: &'a mut CommandExpander,
    pos_count: usize,
    pos_optional_seen: bool,
    pos_many_seen: bool,
    key_many_seen: bool,
    key_names: HashSet<String>,
}

impl<'a> VariantExpander<'a> {
    /// Creates a new `VariantExpander`.
    pub fn new(command_expander: &'a mut CommandExpander) -> Self {
        Self {
            command_expander,
            pos_count: 0,
            pos_optional_seen: false,
            pos_many_seen: false,
            key_many_seen: false,
            key_names: HashSet::new(),
        }
    }

    /// Expands an enum variant as a command parser.
    pub fn expand(mut self, variant: &Variant) -> Result<TokenStream> {
        let ident = &variant.ident;

        if self.command_expander.has_other {
            return Err(Error::new_spanned(
                &variant.ident,
                "command with 'other' must be the last enum variant",
            ));
        }

        // Parse the variant config.
        let VariantConfig { name, other } = parse_variant(variant)?;

        let name = match (name, other) {
            (Some(name), false) => name,
            (None, true) => {
                // `#[command(other)]´ just emits a wildcard arm
                self.command_expander.has_other = true;
                return Ok(quote! {
                    _ => ::core::result::Result::Ok(Self::#ident(command.clone()))
                });
            }

            // parse_variant ensures these invariants hold.
            (Some(_), true) | (None, false) => panic!("must give name or other"),
        };

        if !self.command_expander.command_names.insert(name.clone()) {
            return Err(Error::new_spanned(
                &variant.ident,
                format!("duplicate command name '{name}'"),
            ));
        }

        // Generate the argument parsing code for each variant field.
        let mut statements = Vec::new();
        let mut field_bindings = Vec::new();

        for (index, field) in variant.fields.iter().enumerate() {
            let binding = format_ident!("__field{index}");
            let parse_field = self.expand_field(field)?;
            statements.push(quote! {
                let #binding = #parse_field;
            });
            field_bindings.push((field.ident.as_ref(), binding));
        }

        // Generate the variant construction code, mapping fields to the parsed value bindings.
        let ident = &variant.ident;
        let constructor = match &variant.fields {
            Fields::Unit => quote!(Self::#ident),
            Fields::Unnamed(_) => {
                let fields = field_bindings.iter().map(|(_, binding)| binding);
                quote!(Self::#ident(#(#fields),*))
            }
            Fields::Named(_) => {
                let fields = field_bindings.iter().map(|(field_ident, binding)| {
                    let field_ident = field_ident.expect("named field");
                    quote!(#field_ident: #binding)
                });
                quote!(Self::#ident { #(#fields),* })
            }
        };

        // Emit the generated code for the command match arm.
        Ok(quote! {
            #name => {
                let mut __args = command.consume_args();
                #(#statements)*
                __args.reject_next()?;
                ::core::result::Result::Ok(#constructor)
            }
        })
    }

    /// Expands an enum field as an argument parser.
    fn expand_field(&mut self, field: &Field) -> Result<TokenStream> {
        // Parse the field config.
        let FieldConfig { kind, key, optional, many } = parse_field(field)?;

        // `many` fields must be the last for their kind.
        let many_seen = match kind {
            ArgumentKind::Positional => &mut self.pos_many_seen,
            ArgumentKind::Keyed => &mut self.key_many_seen,
        };
        if *many_seen {
            return Err(Error::new_spanned(
                field,
                "a field can't follow a many field of the same kind",
            ));
        }
        *many_seen |= many;

        // Optional positional fields can't be followed by required fields.
        if kind == ArgumentKind::Positional {
            self.pos_count += 1;
            if !optional && self.pos_optional_seen {
                return Err(Error::new_spanned(
                    field,
                    "a required positional argument can't follow an optional one",
                ));
            }
            self.pos_optional_seen |= optional;
        }

        // Argument keys must be unique.
        if let Some(key) = &key
            && !self.key_names.insert(key.clone())
        {
            return Err(Error::new_spanned(field, format!("duplicate argument key '{key}'")));
        }

        // Determine whether to use the default value for optional fields. For `Option` fields, the
        // generated code should return `None` for missing arguments, otherwise it needs to fall
        // back to the field's default value.
        let default = optional && type_ident(&field.ty).is_none_or(|i| i != "Option");

        // Emit code to parse the field argument(s).
        let label = key
            .clone()
            .or_else(|| field.ident.as_ref().map(|ident| ident.to_string()))
            .unwrap_or_else(|| match kind {
                ArgumentKind::Positional => self.pos_count.to_string(),
                ArgumentKind::Keyed => type_name(&field.ty).unwrap_or_default(),
            });

        Ok(match (kind, many, key) {
            (ArgumentKind::Positional, false, None) => {
                emit_parse_arg(emit_next_pos(), &label, optional, default)
            }
            (ArgumentKind::Keyed, false, Some(key)) => {
                emit_parse_arg(emit_take_key(&key), &label, optional, default)
            }
            (ArgumentKind::Positional, true, None) => emit_collect_pos(&label, optional),
            (ArgumentKind::Keyed, true, None) => emit_collect_key(&label, optional),

            // FieldConfig ensures these invariants hold.
            (ArgumentKind::Positional, _, Some(key)) => panic!("key {key} given for pos"),
            (ArgumentKind::Keyed, false, None) => panic!("no key given"),
            (ArgumentKind::Keyed, true, Some(key)) => panic!("key {key} given for key,many"),
        })
    }
}

/// Parsed config for an enum variant (i.e. command).
struct VariantConfig {
    name: Option<String>,
    other: bool,
}

/// Parses a VariantConfig from an enum variant.
fn parse_variant(variant: &Variant) -> Result<VariantConfig> {
    // Parse the `#[command]` attribute, if any.
    let mut name = None;
    let mut other = false;
    for attr in &variant.attrs {
        if attr.path().is_ident("command") {
            parse_attr_meta(attr, |meta| {
                if meta.path.is_ident("name") {
                    if other {
                        return Err(meta.error("name and other are exclusive"));
                    }
                    name = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if meta.path.is_ident("other") {
                    ensure_bare_meta(&meta)?;
                    if name.is_some() {
                        return Err(meta.error("name and other are exclusive"));
                    }
                    other = true;
                } else {
                    return Err(meta.error("unknown command option"));
                }
                Ok(())
            })?;
        } else if is_helper_attr(attr) {
            return Err(Error::new_spanned(attr, "attribute not valid on enum variant"));
        }
    }

    // Resolve the command name.
    let name = (!other).then(|| name.unwrap_or_else(|| variant.ident.to_string().to_snake_case()));
    assert!(name.is_some() != other);

    // Validate the catch-all variant.
    if other {
        if !matches!(
            &variant.fields,
            Fields::Unnamed(fields) if fields.unnamed.len() == 1
        ) {
            return Err(Error::new_spanned(
                &variant.ident,
                "other command must have exactly one unnamed field",
            ));
        }
        let field = variant.fields.iter().next().expect("checked one field");
        if let Some(attr) = field.attrs.iter().find(|attr| is_helper_attr(attr)) {
            return Err(Error::new_spanned(attr, "attribute not valid on 'other' command field"));
        }
    }

    Ok(VariantConfig { name, other })
}

#[derive(Default)]
struct FieldConfig {
    kind: ArgumentKind,
    key: Option<String>,
    optional: bool,
    many: bool,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum ArgumentKind {
    #[default]
    Positional,
    Keyed,
}

/// Parses a FieldConfig from an enum field.
fn parse_field(field: &Field) -> Result<FieldConfig> {
    // Infer the base config from the field type.
    let mut config = infer_field_config(field);

    // Parse field attributes.
    let mut kind_seen = None;
    for attr in &field.attrs {
        if attr.path().is_ident("arg") {
            parse_attr_meta(attr, |meta| {
                if meta.path.is_ident("pos") {
                    // pos: specifies a positional argument.
                    ensure_bare_meta(&meta)?;
                    config.kind = ArgumentKind::Positional;
                    if kind_seen.replace(config.kind) == Some(ArgumentKind::Keyed) {
                        return Err(Error::new_spanned(attr, "pos and key are exclusive"));
                    }
                } else if meta.path.is_ident("key") {
                    // key: specifies a key=value argument.
                    config.kind = ArgumentKind::Keyed;
                    if kind_seen.replace(config.kind) == Some(ArgumentKind::Positional) {
                        return Err(Error::new_spanned(attr, "pos and key are exclusive"));
                    }
                    config.key = maybe_parse_meta_value::<LitStr>(&meta)?.map(|v| v.value());
                } else if meta.path.is_ident("optional") {
                    // optional: specifies an optional argument.
                    ensure_bare_meta(&meta)?;
                    config.optional = true;
                } else if meta.path.is_ident("many") {
                    // many: specifies a variadic argument.
                    ensure_bare_meta(&meta)?;
                    config.many = true;
                } else {
                    return Err(meta.error("unknown argument option"));
                }
                Ok(())
            })?;
        } else if is_helper_attr(attr) {
            return Err(Error::new_spanned(attr, "attribute not valid on field"));
        } else {
            continue;
        }
    }

    // Infer or validate the key name.
    assert!(!(config.kind == ArgumentKind::Positional && config.key.is_some()));

    if config.kind == ArgumentKind::Keyed {
        // Infer the key name from the field name if not given explicitly.
        if config.key.is_none() && !config.many {
            let Some(ident) = &field.ident else {
                return Err(Error::new_spanned(field, "tuple key fields require key name"));
            };
            config.key = Some(ident.to_string());
        }

        // Key fields with many cannot give key name.
        if config.key.is_some() && config.many {
            return Err(Error::new_spanned(field, "key fields with many cannot give key name"));
        }
    }

    Ok(config)
}

/// Infers the base FieldConfig from the field type.
fn infer_field_config(field: &Field) -> FieldConfig {
    let mut config = FieldConfig::default();

    let Some(ident) = type_ident(&field.ty) else {
        return config;
    };

    match ident.to_string().as_str() {
        "Option" => config.optional = true,
        "Vec" if has_generic_tuple_pair(&field.ty) => {
            config.kind = ArgumentKind::Keyed;
            config.many = true;
        }
        "HashMap" | "BTreeMap" => {
            config.kind = ArgumentKind::Keyed;
            config.many = true;
        }
        "Vec" | "VecDeque" | "HashSet" | "BTreeSet" => config.many = true,
        _ => {}
    };
    config
}

/// Returns whether the type has a single generic `(K, V)` argument.
fn has_generic_tuple_pair(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    let mut arguments = arguments.args.iter();
    let Some(GenericArgument::Type(Type::Tuple(pair))) = arguments.next() else {
        return false;
    };
    if arguments.next().is_some() {
        return false;
    }
    pair.elems.len() == 2
}

/// Parses attribute meta items, calling the given callback for each one.
fn parse_attr_meta(
    attr: &Attribute,
    logic: impl FnMut(ParseNestedMeta<'_>) -> Result<()>,
) -> Result<()> {
    match &attr.meta {
        Meta::Path(_) => Ok(()),
        Meta::List(_) => attr.parse_nested_meta(logic),
        Meta::NameValue(_) => Err(Error::new_spanned(attr, "expected attribute options")),
    }
}

/// Parses an optional `= value` from an attribute meta item.
fn maybe_parse_meta_value<T: Parse>(meta: &ParseNestedMeta<'_>) -> Result<Option<T>> {
    if meta.input.peek(syn::Token![=]) {
        return meta.value()?.parse().map(Some);
    }
    if meta.input.peek(syn::token::Paren) {
        let name = meta.path.get_ident().expect("option name is an identifier");
        return Err(meta.error(format!("{name} values must be specified with `=`")));
    }
    Ok(None)
}

/// Ensures the given attribute meta item is bare, i.e. has no value.
fn ensure_bare_meta(meta: &ParseNestedMeta<'_>) -> Result<()> {
    if meta.input.peek(syn::Token![=]) || meta.input.peek(syn::token::Paren) {
        let name = meta.path.get_ident().expect("option name is an identifier");
        return Err(meta.error(format!("{name} does not accept a value")));
    }
    Ok(())
}

/// Returns the type ident, or None if it could not be determined.
fn type_ident(ty: &Type) -> Option<&Ident> {
    let Type::Path(path) = &ty else {
        return None;
    };
    Some(&path.path.segments.last()?.ident)
}

/// Returns a concise name for a type.
fn type_name(ty: &Type) -> Option<String> {
    type_ident(ty).map(|i| i.to_string())
}

/// Emits an expression for the next positional argument (or None).
fn emit_next_pos() -> TokenStream {
    quote!(__args.next_pos())
}

/// Emits an expression for the next keyed argument (or None).
fn emit_next_key() -> TokenStream {
    quote!(__args.next_key())
}

/// Emits an expression for the given keyed argument (or None).
fn emit_take_key(key: &str) -> TokenStream {
    quote!(__args.take_key(#key))
}

/// Emits a `parse()` call on the given value, with an appropriate error message. Does not
/// emit a trailing ? for the result.
fn emit_parse_value(value: TokenStream, label: &str) -> TokenStream {
    quote! {
        {
            let __value = #value;
            __value.parse().map_err(
                |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                    ::std::format!("invalid value '{__value}' for argument '{}': {err}", #label).into()
                }
            )
        }
    }
}

/// Emits code that requires the given argument Option is Some, otherwise returns an error via `?`.
fn emit_require_arg(arg: TokenStream, label: &str) -> TokenStream {
    quote! {
        #arg.ok_or_else(
            || -> ::std::boxed::Box<dyn ::std::error::Error> {
                ::std::format!("argument '{}' not given", #label).into()
            }
        )?
    }
}

/// Emits code that obtains one argument from an Option<String>, parsing it into the target type. If
/// optional is false, errors if the argument is None. If default is true, unwraps None to the
/// default value (assuming the target field is not an Option<T>).
fn emit_parse_arg(arg: TokenStream, label: &str, optional: bool, default: bool) -> TokenStream {
    if optional {
        let parsed = emit_parse_value(quote!(arg), label);
        let maybe_unwrap_default = default.then_some(quote!(.unwrap_or_default()));
        quote!(#arg.map(|arg| #parsed).transpose()?#maybe_unwrap_default)
    } else {
        let value = emit_require_arg(arg, label);
        let parsed = emit_parse_value(value, label);
        quote!(#parsed?)
    }
}

/// Emits code for an argument iterator that consumes by repeatedly calling next. If optional is
/// false, returns an error if the iterator is empty.
fn emit_arg_iter(next: TokenStream, label: &str, optional: bool) -> TokenStream {
    let iter = quote!(::core::iter::from_fn(|| #next));
    if optional {
        return iter;
    }
    let require_value = emit_require_arg(quote!(__iter.peek()), label);
    quote! {
        {
            let mut __iter = #iter.peekable();
            #require_value;
            __iter
        }
    }
}

/// Emits code that collects all positional arguments.
fn emit_collect_pos(label: &str, optional: bool) -> TokenStream {
    let iter = emit_arg_iter(emit_next_pos(), label, optional);
    let parsed = emit_parse_value(quote!(arg), label);
    quote! {
        #iter
            .map(|arg| #parsed)
            .collect::<::core::result::Result<_, _>>()?
    }
}

/// Emits code that collects all key/value arguments.
fn emit_collect_key(label: &str, optional: bool) -> TokenStream {
    let iter = emit_arg_iter(emit_next_key(), label, optional);
    let parsed_key = emit_parse_value(quote!(key), label);
    let parsed_value = emit_parse_value(quote!(arg), label);
    quote! {
        #iter
            .map(|(key, arg)| {
                #parsed_key.and_then(|key| #parsed_value.map(|value| (key, value)))
            })
            .collect::<::core::result::Result<_, _>>()?
    }
}
