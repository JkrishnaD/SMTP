use tokio::{io::AsyncWriteExt, net::TcpListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listerner = TcpListener::bind("127.0.0.1:2525").await?;
    println!("Starting SMTP server on 2525");

    loop {
        let (mut socket, addr) = listerner.accept().await?;
        println!("Accepted connection from: {}", addr);
        println!("Socket stream: {:?}", socket);

        socket.write_all(b"220 simple-smtp ready\n").await?;
    }
}
