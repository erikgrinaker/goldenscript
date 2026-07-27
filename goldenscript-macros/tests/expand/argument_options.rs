impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "optional" => {
                let mut __args = command.consume_args();
                let __field0 = __args
                    .next_pos()
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "optional"
                                )
                                .into()
                            },
                        )
                    })
                    .transpose()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Optional { optional: __field0 })
            }
            "optional_defaults" => {
                let mut __args = command.consume_args();
                let __field0 = __args
                    .next_pos()
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "positional"
                                )
                                .into()
                            },
                        )
                    })
                    .transpose()?
                    .unwrap_or_default();
                let __field1 = __args
                    .take_key("keyed")
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "keyed"
                                )
                                .into()
                            },
                        )
                    })
                    .transpose()?
                    .unwrap_or_default();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::OptionalDefaults {
                    positional: __field0,
                    keyed: __field1,
                })
            }
            "many" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "values").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "values"
                        )
                        .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                let __field1 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_key()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "properties").into()
                        },
                    )?;
                    __iter
                }
                .map(|(key, arg)| {
                    {
                        let __value = key;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "properties"
                                )
                                .into()
                            },
                        )
                    }
                    .and_then(|key| {
                        {
                            let __value = arg;
                            __value.parse().map_err(
                                |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                    ::std::format!(
                                        "invalid value '{__value}' for argument '{}': {err}",
                                        "properties"
                                    )
                                    .into()
                                },
                            )
                        }
                        .map(|value| (key, value))
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Many { values: __field0, properties: __field1 })
            }
            "optional_many" => {
                let mut __args = command.consume_args();
                let __field0 = ::core::iter::from_fn(|| __args.next_pos())
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "values"
                                )
                                .into()
                            },
                        )
                    })
                    .collect::<::core::result::Result<_, _>>()?;
                let __field1 = ::core::iter::from_fn(|| __args.next_key())
                    .map(|(key, arg)| {
                        {
                            let __value = key;
                            __value.parse().map_err(
                                |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                    ::std::format!(
                                        "invalid value '{__value}' for argument '{}': {err}",
                                        "properties"
                                    )
                                    .into()
                                },
                            )
                        }
                        .and_then(|key| {
                            {
                                let __value = arg;
                                __value.parse().map_err(
                                    |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                        ::std::format!(
                                            "invalid value '{__value}' for argument '{}': {err}",
                                            "properties"
                                        )
                                        .into()
                                    },
                                )
                            }
                            .map(|value| (key, value))
                        })
                    })
                    .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::OptionalMany {
                    values: __field0,
                    properties: __field1,
                })
            }
            "non_generic_collections" => {
                let mut __args = command.consume_args();
                let __field0 = ::core::iter::from_fn(|| __args.next_pos())
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "values"
                                )
                                .into()
                            },
                        )
                    })
                    .collect::<::core::result::Result<_, _>>()?;
                let __field1 = ::core::iter::from_fn(|| __args.next_key())
                    .map(|(key, arg)| {
                        {
                            let __value = key;
                            __value.parse().map_err(
                                |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                    ::std::format!(
                                        "invalid value '{__value}' for argument '{}': {err}",
                                        "properties"
                                    )
                                    .into()
                                },
                            )
                        }
                        .and_then(|key| {
                            {
                                let __value = arg;
                                __value.parse().map_err(
                                    |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                        ::std::format!(
                                            "invalid value '{__value}' for argument '{}': {err}",
                                            "properties"
                                        )
                                        .into()
                                    },
                                )
                            }
                            .map(|value| (key, value))
                        })
                    })
                    .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::NonGenericCollections {
                    values: __field0,
                    properties: __field1,
                })
            }
            name => ::core::result::Result::Err(::std::format!("unknown command '{name}'").into()),
        }
    }
}
