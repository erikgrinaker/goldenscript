impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "scalar" => {
                let mut __args = command.consume_args();
                let __field0 = {
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
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Scalar(__field0))
            }
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
                                    "1"
                                )
                                .into()
                            },
                        )
                    })
                    .transpose()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Optional(__field0))
            }
            "vec" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "1").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "1")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Vec(__field0))
            }
            "vec_deque" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "1").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "1")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::VecDeque(__field0))
            }
            "vec_key_value" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_key()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "Vec").into()
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
                                    "Vec"
                                )
                                .into()
                            },
                        )
                    }
                    .map(|value| (key.to_owned(), value))
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::VecKeyValue(__field0))
            }
            "hash_map" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_key()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "HashMap").into()
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
                                    "HashMap"
                                )
                                .into()
                            },
                        )
                    }
                    .map(|value| (key.to_owned(), value))
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::HashMap(__field0))
            }
            "hash_set" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "1").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "1")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::HashSet(__field0))
            }
            "b_tree_map" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_key()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "BTreeMap").into()
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
                                    "BTreeMap"
                                )
                                .into()
                            },
                        )
                    }
                    .map(|value| (key.to_owned(), value))
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::BTreeMap(__field0))
            }
            "b_tree_set" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "1").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "1")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::BTreeSet(__field0))
            }
            name => ::core::result::Result::Err(::std::format!("unknown command '{name}'").into()),
        }
    }
}
