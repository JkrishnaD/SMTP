use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::parser::parse_command;

mod parser;
mod session;

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

    let mut session = session::Session::new();

    loop {
        line.clear();

        let bytes = reader.read_line(&mut line).await?;

        if bytes == 0 {
            break Ok(());
        }
        let cmd = parse_command(&line);

        session.apply_command(cmd);

        let helo = session.get_helo();
        let main_from = session.get_main_from();
        let recipients = session.get_recipients();

        println!(
            "HELO: {:?}, MAIL_FROM: {:?}, RECIPIENTS: {:?}",
            helo, main_from, recipients
        );
    }
}
