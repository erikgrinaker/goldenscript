impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "unit" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Unit)
            }
            "tuple" => {
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
                let __field1 = {
                    let mut __iter = ::core::iter::from_fn(|| __args.next_pos()).peekable();
                    __iter.peek().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "2").into()
                        },
                    )?;
                    __iter
                }
                .map(|arg| {
                    let __value = arg;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "2")
                            .into()
                    })
                })
                .collect::<::core::result::Result<_, _>>()?;
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Tuple(__field0, __field1))
            }
            "struct" => {
                let mut __args = command.consume_args();
                let __field0 = {
                    let __value = __args.next_pos().ok_or_else(
                        || -> ::std::boxed::Box<dyn ::std::error::Error> {
                            ::std::format!("argument '{}' not given", "name").into()
                        },
                    )?;
                    __value.parse().map_err(|err| -> ::std::boxed::Box<dyn ::std::error::Error> {
                        ::std::format!("invalid value '{__value}' for argument '{}': {err}", "name")
                            .into()
                    })
                }?;
                let __field1 = {
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
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Struct { name: __field0, values: __field1 })
            }
            name => ::core::result::Result::Err(::std::format!("unknown command '{name}'").into()),
        }
    }
}
