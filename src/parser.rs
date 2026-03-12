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
    Unknown,
}

// Parses a command from a line of input
pub fn parse_command(line: &str) -> Command {
    let input = line.trim();

    // Parse the command based on the input line
    if input.starts_with("EHLO") {
        let domain = input.strip_prefix("EHLO ").unwrap_or(" ");
        Command::Ehlo(domain.to_string())
    } else if input.starts_with("STARTTLS") {
        Command::StartTls
    } else if input.starts_with("AUTH") {
        let email = input.strip_prefix("AUTH ").unwrap_or(" ");
        Command::Auth(email.to_string())
    } else if input.starts_with("HELO") {
        let domain = input.strip_prefix("HELO ").unwrap_or(" ");
        Command::Helo(domain.to_string())
    } else if input.starts_with("MAIL FROM:") {
        let email = input.strip_prefix("MAIL FROM:").unwrap_or(" ");
        Command::MailFrom(email.to_string())
    } else if input.starts_with("RCPT TO:") {
        let email = input.strip_prefix("RCPT TO:").unwrap_or(" ");
        Command::RcptTo(email.to_string())
    } else if input.starts_with("DATA") {
        Command::Data
    } else if input.starts_with("VRFY") {
        let email = input.strip_prefix("VRFY ").unwrap_or(" ");
        Command::Vrfy(email.to_string())
    } else if input.starts_with("NOOP") {
        Command::Noop
    } else if input.starts_with("HELP") {
        Command::Help
    } else if input.starts_with("LIST") {
        let email = input.strip_prefix("LIST ").unwrap_or(" ");
        Command::List(email.to_string())
    } else if input.starts_with("QUIT") {
        Command::Quit
    } else if input.starts_with("RSET") {
        Command::Rset
    } else {
        Command::Unknown
    }
}
