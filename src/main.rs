use diesel::r2d2::{ConnectionManager, Pool};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::{config::CONFIG, response::Response, storage::Store};

mod config;
mod models;
mod parser;
mod response;
mod schema;
mod session;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // starting a tcp listener on port 2525
    let listerner = TcpListener::bind("127.0.0.1:2525").await?;
    println!("Starting SMTP server on 2525");

    // getting the database url from the config and creating a connection pool
    let database_url = &CONFIG.db_url;
    let manager = ConnectionManager::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create a database connection");
    let store = Store::new(pool);

    loop {
        let store = store.clone();
        // accepting a new connection and getting
        // the socket stream and address from the listener
        let (socket, addr) = listerner.accept().await?;
        println!("Accepted connection from: {}", addr);

        // spawning a new task to handle the client
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store).await {
                eprintln!("Client {} error: {}", addr, e);
            }
        });
    }
}

// the function that handles the communication with the client
async fn handle_client(
    mut socket: TcpStream,
    store: Store,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Socket stream: {:?}", socket);

    // writing the initial response to the client
    socket.write_all(b"220 simple-smtp ready\r\n").await?;

    // getting the reader and the writer from the socket
    let (reader, mut writer) = socket.into_split();

    // getting the buffered reader from the reader
    let mut reader = BufReader::new(reader);
    let mut line = String::new(); // creating a new string to hold each line

    // creating a new session for each client
    let mut session = session::Session::new();

    loop {
        // clearing the line buffer before reading a new line
        line.clear();
        reader.read_line(&mut line).await?;

        // having the response based on the session state
        let response = session.handle_session(&line, &store);

        // based on the response, write the message or close the connection
        match response {
            Response::Message(msg) => {
                writer.write_all(msg.as_bytes()).await?;
            }
            Response::Close(msg) => {
                writer.write_all(msg.as_bytes()).await?;
                break Ok(());
            }
            Response::None => {}
        }
    }
}
