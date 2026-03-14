use crate::{
    parser::{Command, parse_command},
    response::Response,
    storage::Store,
};

// Represents a session with an SMTP client
pub struct Session {
    ehlo: Option<String>,
    helo: Option<String>,
    username: Option<String>,
    password: Option<String>,
    tls_active: bool,
    authenticated: bool,
    mail_from: Option<String>,
    recipients: Vec<String>,
    state: SessionState,
    buffer: String,
}

// Session state enum to track the current state of the session
#[derive(Debug)]
pub enum SessionState {
    Command,
    AuthUsername,
    AuthPassword,
    Data,
}

// Implements for the Session struct
impl Session {
    // Creates a new Session instance
    pub fn new() -> Session {
        Session {
            ehlo: None,
            helo: None,
            tls_active: false,
            authenticated: false,
            username: None,
            password: None,
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
            Command::StartTls => self.handle_start_tls(),
            Command::Auth(mechanism) => self.handle_auth(mechanism),
            Command::Helo(domain) => self.handle_helo(domain),
            Command::MailFrom(email) => self.handle_mail_from(email),
            Command::RcptTo(email) => self.handle_rcpt_to(email),
            Command::Data => self.handle_data_start(),
            Command::List(email) => self.handle_list(email, store).await,
            Command::Vrfy(email) => self.handle_vrfy(email, store).await,
            Command::Noop => self.handle_noop(),
            Command::Help => self.handle_help(),
            Command::Rset => self.handle_rset(),
            Command::Quit => Response::Close(format!("221 Bye\r\n")),
        }
    }

    pub fn handle_ehlo(&mut self, domain: String) -> Response {
        self.ehlo = Some(domain.clone());

        Response::Message(format!(
            "250-{}\r\n\
        250-PIPELINING\r\n\
        250-SIZE 10485760\r\n\
        250-8BITMIME\r\n\
        250-STARTTLS\r\n\
        250 AUTH LOGIN PLAIN\r\n",
            domain
        ))
    }

    pub fn handle_start_tls(&mut self) -> Response {
        if self.tls_active {
            Response::Message("454 TLS already active\r\n".into())
        } else {
            Response::StartTls
        }
    }

    pub fn set_tls_state(&mut self, active: bool) {
        self.tls_active = active;
    }

    pub fn handle_auth(&mut self, mechanism: String) -> Response {
        if !self.tls_active {
            return Response::Message("538 Encryption required for authentication\r\n".into());
        }

        if mechanism.to_ascii_uppercase() != "LOGIN" {
            return Response::Message("504 Only Auth LOGIN is supported\r\n".to_string());
        }

        self.state = SessionState::AuthUsername;
        self.username = None;
        self.password = None;
        Response::Message("334 VXNlcm5hbWU6:\r\n".into())
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
            SessionState::Command => {
                let cmd = parse_command(line);
                match cmd {
                    Ok(c) => self.apply_command(c, store).await,
                    Err(err) => Response::Message(format!("500 {}\r\n", err)),
                }
            }
            SessionState::AuthUsername => self.handle_auth_username(line, store).await,
            SessionState::AuthPassword => self.handle_auth_password(line, store).await,
            SessionState::Data => self.handle_data(line, store).await,
        }
    }

    pub async fn handle_auth_username(&mut self, line: &str, store: &Store) -> Response {
        let email = line.trim().to_string();

        match store.get_user_by_email(&email).await {
            Ok(Some(user)) => {
                self.username = Some(user.email);
                self.state = SessionState::AuthPassword;
                Response::Message("334 Password required\r\n".to_string())
            }
            Ok(None) => {
                self.state = SessionState::Command;
                Response::Message("535 Authentication credentials invalid\r\n".into())
            }
            Err(_) => {
                self.state = SessionState::Command;
                Response::Message("454 Temporary authentication failure\r\n".into())
            }
        }
    }

    pub async fn handle_auth_password(&mut self, line: &str, store: &Store) -> Response {
        let email = match &self.username {
            Some(e) => e.clone(),
            None => return Response::Message("535 Authentication credentials invalid\r\n".into()),
        };
        let password = line.trim().to_string();

        match store.verify_password(&email, &password).await {
            Ok(true) => {
                self.password = Some(password);
                self.authenticated = true;
                self.state = SessionState::Command;
                Response::Message("235 Authentication successful\r\n".to_string())
            }
            Ok(false) => {
                self.state = SessionState::Command;
                Response::Message("535 Authentication credentials invalid\r\n".into())
            }
            Err(_) => {
                self.state = SessionState::Command;
                Response::Message("454 Temporary authentication failure\r\n".into())
            }
        }
    }

    // Handle the HELO command
    pub fn handle_helo(&mut self, domain: String) -> Response {
        self.helo = Some(domain);
        Response::Message(format!(
            "250 Hello {}, pleased to meet you\r\n",
            self.helo.as_ref().unwrap()
        ))
    }

    // Handle the MAIL FROM command
    pub fn handle_mail_from(&mut self, email: String) -> Response {
        if self.ehlo.is_none() && self.helo.is_none() {
            return Response::Message("503 Send HELO/EHLO first\r\n".into());
        }

        if self.tls_active == false {
            return Response::Message("530 Tls handshake required\r\n".into());
        }

        if self.authenticated == false {
            return Response::Message("530 Authentication required\r\n".into());
        }

        self.mail_from = Some(email);
        Response::Message(format!("250 OK\r\n",))
    }

    // Handle the RCPT TO command
    pub fn handle_rcpt_to(&mut self, email: String) -> Response {
        if self.tls_active == false {
            return Response::Message("530 Tls handshake required\r\n".into());
        }

        if self.authenticated == false {
            return Response::Message("530 Authentication required\r\n".into());
        }

        if self.mail_from.is_none() {
            Response::Message(format!("503 Error: Need From mail\r\n"))
        } else {
            self.recipients.push(email);
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

    // Handle the NOOP command
    pub fn handle_noop(&mut self) -> Response {
        // It does nothing but respond with success and keep the session open
        Response::Message(format!("250 OK\r\n"))
    }

    pub async fn handle_vrfy(&mut self, email: String, store: &Store) -> Response {
        let email = store.verify_email(email).await;
        match email {
            Ok(true) => Response::Message("250 User exists\r\n".into()),
            Ok(false) => Response::Message("550 No such user\r\n".into()),
            Err(_) => Response::Message("451 Temporary error\r\n".into()),
        }
    }

    pub fn handle_help(&mut self) -> Response {
        let commands = "EHLO <domain>\r\n\
            HELO <user>\r\n\
            STARTTLS - for TLS encryption\r\n\
            AUTH: <email> - for authentication\r\n\
            MAIL FROM: <email>\r\n\
            RCPT TO: <email>\r\n\
            DATA: <message>\r\n\
            LIST: <user>\r\n\
            NOOP\r\n\
            RSET\r\n\
            HELP - for help\r\n\
            QUIT";
        Response::Message(format!("250 Help: {}\r\n", commands))
    }

    // Handle the RSET command
    pub fn handle_rset(&mut self) -> Response {
        // It resets the session state to the initial state, clearing any pending mail or recipients
        self.mail_from = None;
        self.recipients.clear();
        self.buffer.clear();

        self.state = SessionState::Command;

        Response::Message(format!("250 Reset OK\r\n"))
    }
}
