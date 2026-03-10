use diesel::{Connection, SqliteConnection};

use crate::{
    parser::{Command, parse_command},
    response::Response,
    storage::Store,
};

// Represents a session with an SMTP client
pub struct Session {
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
    HeloRecieved,
    MailFromRecieved,
    RcptRecieved,
    Data,
}

// Implements for the Session struct
impl Session {
    // Creates a new Session instance
    pub fn new() -> Session {
        Session {
            helo: None,
            mail_from: None,
            recipients: Vec::new(),
            state: SessionState::Command,
            buffer: String::new(),
        }
    }

    // Applies a command to the session and returns a response
    pub fn apply_command(&mut self, cmd: Command, conn: &mut SqliteConnection) -> Response {
        match cmd {
            Command::Helo(domain) => self.handle_helo(domain),
            Command::MailFrom(email) => self.handle_mail_from(email),
            Command::RcptTo(email) => self.handle_rcpt_to(email),
            Command::Data => self.handle_data_start(),
            Command::List(email) => self.handle_list(conn, email),
            Command::Quit => Response::Close(format!("221 Bye\r\n")),
            Command::Unknown => Response::Close(format!("505 Unknown Command\r\n")),
        }
    }

    // Handles data input during the DATA state and returns a response
    pub fn handle_data(&mut self, line: &str, conn: &mut SqliteConnection) -> Response {
        if line.trim() == "." {
            let email = self.mail_from.clone().unwrap();

            let res = conn.transaction::<(), diesel::result::Error, _>(|conn| {
                println!("Database Transaction Started");
                println!("Saving email from: {}", email);

                Store::save_email(conn, email, self.recipients.clone(), self.buffer.clone())?;
                println!(
                    "Successfully saved email and {} recipients",
                    self.recipients.len()
                );
                Ok(())
            });

            match res {
                Ok(_) => {
                    self.buffer.clear();
                    self.recipients.clear();
                    self.state = SessionState::Command;
                    Response::Message(format!("250 OK\r\n"))
                }
                Err(_) => Response::Close(format!("500 Internal Server Error\r\n")),
            }
        } else {
            self.buffer.push_str(&line);
            Response::None
        }
    }

    pub fn handle_list(&mut self, conn: &mut SqliteConnection, user: String) -> Response {
        let res = Store::get_mails(conn, user);
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
            Err(_) => Response::Close(format!("500 Internal Server Error\r\n")),
        }
    }

    // Based on the request we got, update the session state and return a response
    pub fn handle_session(&mut self, line: &str, store: &Store) -> Response {
        let mut conn = store.pool.get().expect("Failed to get Database Connection");
        match self.state {
            SessionState::Command
            | SessionState::HeloRecieved
            | SessionState::MailFromRecieved
            | SessionState::RcptRecieved => {
                let cmd = parse_command(line);
                self.apply_command(cmd, &mut conn)
            }
            SessionState::Data => self.handle_data(line, &mut conn),
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
}
