#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidCommand,
    MissingArguments(&'static str),
    InvalidSyntax(&'static str),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidCommand => write!(f, "Invalid command"),
            ParseError::MissingArguments(msg) => write!(f, "Missing argumrnt: {}", msg),
            ParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
        }
    }
}
