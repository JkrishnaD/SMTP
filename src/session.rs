use crate::{
    parser::{Command, parse_command},
    response::Response,
    storage::Store,
};

// Represents a session with an SMTP client
pub struct Session {
    ehlo: Option<String>,
    helo: Option<String>,
    mail_from: Option<String>,
    recipients: Vec<String>,
    state: SessionState,
    buffer: String,
}

// Session state enum to track the current state of the session
#[derive(Debug)]
pub enum SessionState {
    Command,
    EhloVerified,
    HeloRecieved,
    MailFromRecieved,
    RcptRecieved,
    Data,
    Reset,
}

// Implements for the Session struct
impl Session {
    // Creates a new Session instance
    pub fn new() -> Session {
        Session {
            ehlo: None,
            helo: None,
            mail_from: None,
            recipients: Vec::new(),
            state: SessionState::Command,
            buffer: String::new(),
        }
    }

    // Applies a command to the session and returns a response
    pub async fn apply_command(&mut self, cmd: Command, store: &Store) -> Response {
        match cmd {
            Command::Ehlo(domain) => self.handle_ehlo(domain),
            Command::Helo(domain) => self.handle_helo(domain),
            Command::MailFrom(email) => self.handle_mail_from(email),
            Command::RcptTo(email) => self.handle_rcpt_to(email),
            Command::Data => self.handle_data_start(),
            Command::List(email) => self.handle_list(email, store).await,
            Command::Reset => self.handle_reset(),
            Command::Quit => Response::Close(format!("221 Bye\r\n")),
            Command::Unknown => Response::Message(format!("500 Unknown Command\r\n")),
        }
    }

    pub fn handle_ehlo(&mut self, domain: String) -> Response {
        self.ehlo = Some(domain);
        self.state = SessionState::EhloVerified;
        Response::Message("250-localhost\r\n250-SIZE 10485760\r\n250 PIPELINING\r\n".to_string())
    }

    // Handles data input during the DATA state and returns a response
    pub async fn handle_data(&mut self, line: &str, store: &Store) -> Response {
        if line.trim() == "." {
            let email = self.mail_from.clone().unwrap_or_default();
            let recipients = self.recipients.clone();
            let buffer = self.buffer.clone();

            match store.save_emails_async(email, recipients, buffer).await {
                Ok(_) => {
                    self.buffer.clear();
                    self.recipients.clear();
                    self.state = SessionState::Command;
                    Response::Message(format!("250 OK\r\n"))
                }
                Err(_) => Response::Close(format!("500 Internal Server Error\r\n")),
            }
        } else {
            self.buffer.push_str(line);
            Response::None
        }
    }

    pub async fn handle_list(&mut self, user: String, store: &Store) -> Response {
        let res = store
            .get_mails_async(user)
            .await
            .map_err(|e| Response::Close(format!("500 Internal Server Error: {}\r\n", e)));

        match res {
            Ok(mails) => {
                if mails.is_empty() {
                    return Response::Message(format!("250 No mails found\r\n"));
                }

                let mut response = String::new();

                for mail in mails {
                    response.push_str(&format!("MAIL: {} {}\r\n", mail.id, mail.sender));
                }
                response.push_str("250 OK\r\n");

                Response::Message(response)
            }
            Err(err) => err,
        }
    }

    // Based on the request we got, update the session state and return a response
    pub async fn handle_session(&mut self, line: &str, store: &Store) -> Response {
        match self.state {
            SessionState::Command
            | SessionState::Reset
            | SessionState::EhloVerified
            | SessionState::HeloRecieved
            | SessionState::MailFromRecieved
            | SessionState::RcptRecieved => {
                let cmd = parse_command(line);
                self.apply_command(cmd, store).await
            }
            SessionState::Data => self.handle_data(line, store).await,
        }
    }

    // Handle the HELO command
    pub fn handle_helo(&mut self, domain: String) -> Response {
        self.helo = Some(domain);
        self.state = SessionState::HeloRecieved;
        Response::Message(format!(
            "250 Hello {}, pleased to meet you\r\n",
            self.helo.as_ref().unwrap()
        ))
    }

    // Handle the MAIL FROM command
    pub fn handle_mail_from(&mut self, email: String) -> Response {
        if self.ehlo.is_none() || self.helo.is_none() {
            return Response::Message(format!("503 Error: Need EHLO and HELO first\r\n"));
        };
        self.mail_from = Some(email);
        self.state = SessionState::MailFromRecieved;
        Response::Message(format!("250 OK\r\n",))
    }

    // Handle the RCPT TO command
    pub fn handle_rcpt_to(&mut self, email: String) -> Response {
        if self.mail_from.is_none() {
            Response::Message(format!("503 Error: Need From mail\r\n"))
        } else {
            self.recipients.push(email);
            self.state = SessionState::RcptRecieved;
            Response::Message(format!("250 OK\r\n",))
        }
    }

    // Handle the DATA command
    pub fn handle_data_start(&mut self) -> Response {
        if self.mail_from.is_none() {
            return Response::Message(format!("503 Error: Need From mail\r\n"));
        } else if self.recipients.is_empty() {
            return Response::Message(format!("503 Error: Need Rcpt mail\r\n"));
        } else {
            self.state = SessionState::Data;
            Response::Message(format!("354 Start mail input\r\n"))
        }
    }

    pub fn handle_reset(&mut self) -> Response {
        self.mail_from = None;
        self.recipients.clear();
        self.buffer.clear();

        if self.helo.is_some() {
            self.state = SessionState::EhloVerified;
        } else {
            self.state = SessionState::Command;
        }

        Response::Message(format!("250 Reset OK\r\n"))
    }
}
