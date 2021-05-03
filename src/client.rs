use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::BufMut;

use std::env;
use std::error::Error;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".into())
        .parse()?;
    let subdomain: String = env::args()
        .nth(2)
        .unwrap_or_else(|| "service".into());

    let mut socket = TcpStream::connect(remote_addr).await?;
    let mut stdin = io::stdin();

    loop {
        let mut buf = vec![0; 1024];
        let n = stdin
            .read(&mut buf)
            .await
            .expect("failed to read data from stdin");

        if n == 0 {
            continue;
        }

        let mut bytes = vec![];
        bytes.put_u32(subdomain.len() as u32);
        bytes.put(subdomain.as_bytes());
        bytes.put_u32(n as u32);
        bytes.put(&buf[0..n]);

        socket
            .write_all(&bytes)
            .await
            .expect("failed to write data to socket");
    }
}
