use std::time::Duration;

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

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
    tokio::time::sleep(Duration::from_secs(3)).await;
    Ok(())
}
