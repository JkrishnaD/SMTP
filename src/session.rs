use crate::{parser::Command, response::Response};

pub struct Session {
    helo: Option<String>,
    mail_from: Option<String>,
    recipients: Vec<String>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            helo: None,
            mail_from: None,
            recipients: Vec::new(),
        }
    }

    pub fn apply_command(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::Helo(domain) => {
                self.helo = Some(domain.clone());
                Response::Message(format!("250 Hello {}\r\n", domain))
            }
            Command::MailFrom(email) => {
                self.mail_from = Some(email.clone());
                Response::Message(format!("250 {} OK\r\n", email))
            }
            Command::RcptTo(email) => {
                if self.mail_from.is_none() {
                    Response::Message(format!("503 Error: Need From mail\r\n"))
                } else {
                    self.recipients.push(email.clone());
                    Response::Message(format!("250 {} OK\r\n", email))
                }
            }
            Command::Data => Response::Message(format!("354 Start mail input\r\n")),
            Command::Quit => Response::Close(format!("221 Bye\r\n")),
            Command::Unknown => Response::Close(format!("505 Unknown Command\r\n")),
        }
    }
}
