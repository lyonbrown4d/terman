#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ProcessCommandMode {
    #[default]
    Full,
    Name,
}

impl ProcessCommandMode {
    pub(crate) fn toggle(&mut self) {
        *self = match self {
            Self::Full => Self::Name,
            Self::Name => Self::Full,
        };
    }

    pub(crate) fn header(self) -> &'static str {
        match self {
            Self::Full => "COMMAND",
            Self::Name => "NAME",
        }
    }
}