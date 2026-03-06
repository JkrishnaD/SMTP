pub enum Command {
    Helo(String),
    MailFrom(String),
    RcptTo(String),
    Data,
    Quit,
    Unknown,
}

pub fn parse_command(line: &str) -> Command {
    let input = line.trim();

    if input.starts_with("HELO") {
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
    } else if input.starts_with("QUIT") {
        Command::Quit
    } else {
        Command::Unknown
    }
}
