use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::parser::{Command, parse_command};

mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listerner = TcpListener::bind("127.0.0.1:2525").await?;
    println!("Starting SMTP server on 2525");

    loop {
        let (socket, addr) = listerner.accept().await?;
        println!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            let _ = handle_client(socket).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    println!("Socket stream: {:?}", socket);

    socket.write_all(b"220 simple-smtp ready\n").await?;

    let (reader, _writer) = socket.into_split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        let bytes = reader.read_line(&mut line).await?;

        if bytes == 0 {
            break Ok(());
        }
        let cmd = parse_command(&line);
        match cmd {
            Command::Helo(domain) => {
                println!("HELO from {}", domain)
            }
            Command::MailFrom(email) => {
                println!("Sender: {}", email);
            }
            Command::RcptTo(email) => {
                println!("Recipient: {}", email);
            }
            Command::Data => {
                println!("Start recieving msg")
            }
            Command::Quit => {
                return Ok(());
            }
            Command::Unknown => {
                return Err("Unknown command".into());
            }
        }
    }
}
