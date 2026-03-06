use crate::parser::Command;

pub struct Session {
    helo: Option<String>,
    main_from: Option<String>,
    recipients: Vec<String>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            helo: None,
            main_from: None,
            recipients: Vec::new(),
        }
    }

    pub fn apply_command(&mut self, cmd: Command) {
        match cmd {
            Command::Helo(domain) => {
                self.helo = Some(domain);
            }
            Command::MailFrom(email) => {
                self.main_from = Some(email);
            }
            Command::RcptTo(email) => {
                self.recipients.push(email);
            }
            Command::Data => {
                println!("Ready to recieve email body")
            }
            Command::Quit => {
                println!("Session ending")
            }
            Command::Unknown => {
                println!("Unknown SMTP command")
            }
        }
    }

    pub fn get_helo(&self) -> Option<&str> {
        self.helo.as_deref()
    }

    pub fn get_main_from(&self) -> Option<&str> {
        self.main_from.as_deref()
    }

    pub fn get_recipients(&self) -> &[String] {
        &self.recipients
    }
}
