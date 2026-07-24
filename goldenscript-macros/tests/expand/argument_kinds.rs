impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "arguments" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "positional").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "positional"
                        )
                        .into()
                    })
                }?;
                let __field1 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "explicit_positional").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "explicit_positional"
                        )
                        .into()
                    })
                }?;
                let __field2 = {
                    let __value = __args.take_key("keyed").ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "keyed").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "keyed"
                        )
                        .into()
                    })
                }?;
                let __field3 = __args
                    .take_key("renamed")
                    .map(|arg| {
                        let __value = arg;
                        __value.parse().map_err(
                            |err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                                ::std::format!(
                                    "invalid value '{__value}' for argument '{}': {err}",
                                    "renamed"
                                )
                                .into()
                            },
                        )
                    })
                    .transpose()?;
                let __field4 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "rest").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "rest")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                let __field5 = {
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
                    .map(|value| (key.to_owned(), value))
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Arguments {
                    positional: __field0,
                    explicit_positional: __field1,
                    keyed: __field2,
                    named: __field3,
                    rest: __field4,
                    properties: __field5,
                })
            }
            "tuple_key" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let __value = __args.take_key("value").ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "value").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "value"
                        )
                        .into()
                    })
                }?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::TupleKey(__field0))
            }
            "tuple_mixed" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let __value = __args.take_key("first").ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "first").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "first"
                        )
                        .into()
                    })
                }?;
                let __field1 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "1").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "1")
                            .into()
                    })
                }?;
                let __field2 = {
                    let __value = __args.take_key("second").ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "second").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "second"
                        )
                        .into()
                    })
                }?;
                let __field3 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "2").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "2")
                            .into()
                    })
                }?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::TupleMixed(__field0, __field1, __field2, __field3))
            }
            "key_many_then_positional" => {
                let mut __args = command.consume_args();
                let __field0 = {
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
                    .map(|value| (key.to_owned(), value))
                })
                .collect::<::core::result::Result<_, _>>()?;
                let __field1 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "value").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!(
                            "invalid value '{__value}' for argument '{}': {err}",
                            "value"
                        )
                        .into()
                    })
                }?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::KeyManyThenPositional {
                    properties: __field0,
                    value: __field1,
                })
            }
            name => ::core::result::Result::Err(::std::format!("unknown command '{name}'").into()),
        }
    }
}
