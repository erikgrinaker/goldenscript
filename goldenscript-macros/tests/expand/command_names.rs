impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "error" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Error)
            }
            "http_request" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::HTTPRequest)
            }
            "get_url_value" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::GetURLValue)
            }
            "renamed" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Other)
            }
            name => ::core::result::Result::Err(::std::format!("unknown command '{name}'").into()),
        }
    }
}
