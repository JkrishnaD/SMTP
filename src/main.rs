use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::{parser::parse_command, response::Response};

mod parser;
mod response;
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

    let (reader, mut writer) = socket.into_split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let mut session = session::Session::new();

    loop {
        line.clear();
        reader.read_line(&mut line).await?;

        let cmd = parse_command(&line);

        let response = session.apply_command(cmd);
        match response {
            Response::Message(msg) => {
                writer.write_all(msg.as_bytes()).await?;
            }
            Response::Close(msg) => {
                writer.write_all(msg.as_bytes()).await?;
                break Ok(());
            }
        }
    }
}
