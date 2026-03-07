use crate::{parser::Command, response::Response};

pub struct Session {
    helo: Option<String>,
    mail_from: Option<String>,
    recipients: Vec<String>,
    pub state: SessionState,
    buffer: String,
}

#[derive(Debug)]
pub enum SessionState {
    Command,
    Data,
}

impl Session {
    pub fn new() -> Session {
        Session {
            helo: None,
            mail_from: None,
            recipients: Vec::new(),
            state: SessionState::Command,
            buffer: String::new(),
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
            Command::Data => {
                self.state = SessionState::Data;
                Response::Message(format!("354 Start mail input\r\n"))
            }
            Command::Quit => Response::Close(format!("221 Bye\r\n")),
            Command::Unknown => Response::Close(format!("505 Unknown Command\r\n")),
        }
    }

    pub fn handle_data(&mut self, line: &str) -> Response {
        if line.trim() == "<CRFL>.<CRFL>" {
            println!("EMAIL:\n{}", self.buffer);
            self.buffer.clear();

            self.state = SessionState::Command;
            Response::Message(format!("250 OK\r\n"))
        } else {
            self.buffer.push_str(&line);
            Response::None
        }
    }
}
