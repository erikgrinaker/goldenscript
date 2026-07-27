impl ::core::convert::TryFrom<&::goldenscript::Command> for Command {
    type Error = ::std::boxed::Box<dyn ::std::error::Error>;
    fn try_from(
        command: &::goldenscript::Command,
    ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
        match command.name.as_str() {
            "known" => {
                let mut __args = command.consume_args();
                __args.reject_next()?;
                ::core::result::Result::Ok(Self::Known)
            }
            _ => ::core::result::Result::Ok(Self::Other(command.clone())),
        }
    }
}
