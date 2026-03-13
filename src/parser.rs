use crate::error::ParseError;

// All the commands supported by the SMTP parser
pub enum Command {
    Ehlo(String),
    StartTls,
    Auth(String),
    Helo(String),
    MailFrom(String),
    RcptTo(String),
    Data,
    List(String),
    Vrfy(String),
    Noop,
    Help,
    Quit,
    Rset,
}

// Parses a command from a line of input
pub fn parse_command(input: &str) -> Result<Command, ParseError> {
    let input = input.trim();

    // Split the input into command and arguments
    let mut parts = input.splitn(2, ' ');
    // considering the first part as the command and converting it to uppercase
    let cmd = parts.next().unwrap_or("").to_ascii_uppercase();
    // the rest as the arguments
    let args = parts.next().unwrap_or("").trim();

    match cmd.as_str() {
        "EHLO" => {
            if args.is_empty() {
                Err(ParseError::MissingArguments("EHLO requires domain"))
            } else {
                Ok(Command::Ehlo(args.to_string()))
            }
        }
        "STARTTLS" => Ok(Command::StartTls),
        "AUTH" => match args.to_ascii_uppercase().as_str() {
            "LOGIN" | "PLAIN" => Ok(Command::Auth(args.to_string())),
            _ => Err(ParseError::InvalidCommand),
        },
        "HELO" => Ok(Command::Helo(args.to_string())),
        "MAIL" => {
            let upper_args = args.to_ascii_uppercase();
            if upper_args.starts_with("FROM:") {
                let addr = args[5..].trim();

                if addr.is_empty() {
                    return Err(ParseError::MissingArguments("MAIL FROM requires address"));
                }
                Ok(Command::MailFrom(
                    addr.trim().trim_matches(&['<', '>']).to_string(),
                ))
            } else {
                Err(ParseError::InvalidSyntax("MAIL must use FROM:"))
            }
        }
        "RCPT" => {
            let upper_args = args.to_ascii_uppercase();
            if upper_args.starts_with("TO:") {
                let addr = upper_args[3..].trim();
                if addr.is_empty() {
                    return Err(ParseError::MissingArguments("RCPT TO requires address"));
                }
                Ok(Command::RcptTo(
                    addr.trim().trim_matches(&['<', '>']).to_string(),
                ))
            } else {
                Err(ParseError::InvalidSyntax("RCPT must use TO:"))
            }
        }
        "DATA" => Ok(Command::Data),
        "VRFY" => Ok(Command::Vrfy(args.to_string())),
        "NOOP" => Ok(Command::Noop),
        "HELP" => Ok(Command::Help),
        "LIST" => Ok(Command::List(args.to_string())),
        "QUIT" => Ok(Command::Quit),
        "RSET" => Ok(Command::Rset),
        _ => Err(ParseError::InvalidCommand),
    }
}
