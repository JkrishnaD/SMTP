
// All the responses supported by the SMTP parser
pub enum Response {
    Message(String),
    Close(String),
    StartTls,
    None,
}
