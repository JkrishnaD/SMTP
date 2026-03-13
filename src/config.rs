use std::env;

use once_cell::sync::Lazy;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

pub struct Config {
    pub db_url: String,
}

// global config for env variables
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().ok();
    Config {
        db_url: env::var("DATABASE_URL").expect("DATABASE_URl not found"),
    }
});

pub enum Connection {
    Tcp(TcpStream),
    Tls(TlsStream<TcpStream>),
    None,
}

impl Connection {
    pub async fn write_all(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        match self {
            Connection::Tcp(s) => s.write_all(buf).await,
            Connection::Tls(s) => s.write_all(buf).await,
            Connection::None => Ok(()),
        }
    }

    pub async fn read_line(&mut self, buf: &mut String) -> tokio::io::Result<usize> {
        match self {
            Connection::Tcp(s) => {
                let mut reader = BufReader::new(s);
                reader.read_line(buf).await
            }
            Connection::Tls(s) => {
                let mut reader = BufReader::new(s);
                reader.read_line(buf).await
            }
            Connection::None => Ok(0),
        }
    }
}
