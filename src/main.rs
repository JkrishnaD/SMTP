use std::mem::replace;

use diesel::r2d2::{ConnectionManager, Pool};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

use crate::{
    api::create_router,
    config::{CONFIG, Connection},
    response::Response,
    storage::Store,
    tls::load_tls_config,
};

mod api;
mod config;
mod error;
mod models;
mod parser;
mod response;
mod schema;
mod session;
mod storage;
mod tls;

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

    let tls_config_load = load_tls_config()?;
    let tls_acceptor = TlsAcceptor::from(tls_config_load);

    let app = create_router(store.clone());

    tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    loop {
        let store = store.clone();
        let tls_acceptor = tls_acceptor.clone();
        // accepting a new connection and getting
        // the socket stream and address from the listener
        let (socket, addr) = listerner.accept().await?;
        println!("Accepted connection from: {}", addr);

        // spawning a new task to handle the client
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store, tls_acceptor).await {
                eprintln!("Client {} error: {}", addr, e);
            }
        });
    }
}

// the function that handles the communication with the client
async fn handle_client(
    socket: TcpStream,
    store: Store,
    tls_acceptor: TlsAcceptor,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Socket stream: {:?}", socket);

    let mut conn = Connection::Tcp(socket);
    // writing the initial response to the client
    conn.write_all(b"220 simple-smtp ready\r\n").await?;

    // creating a new session for each client
    let mut session = session::Session::new();
    let mut line = String::new(); // creating a new string to hold each line

    loop {
        // clearing the line buffer before reading a new line
        line.clear();
        conn.read_line(&mut line).await?;

        // having the response based on the session state
        let response = session.handle_session(&line, &store).await;

        // based on the response, write the message or close the connection
        match response {
            Response::Message(msg) => {
                conn.write_all(msg.as_bytes()).await?;
            }
            Response::StartTls => {
                conn.write_all(b"220 Ready to start TLS\r\n").await?;
                if let Connection::Tcp(s) = replace(&mut conn, Connection::None) {
                    // accept TLS connection
                    let tls_stream = tls_acceptor.accept(s).await?;
                    // update the connection to TLS from Tcp
                    conn = Connection::Tls(Box::new(tls_stream));
                    // update the session TLS state
                    session.set_tls_state(true);
                    conn.write_all(b"Tls handshake successful!\r\n").await?;
                }
            }
            Response::Close(msg) => {
                conn.write_all(msg.as_bytes()).await?;
                break Ok(());
            }
            Response::None => {}
        }
    }
}
