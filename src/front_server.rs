use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use bytes::{Bytes, Buf};
use once_cell::sync::Lazy;

use std::net::SocketAddr;
use std::env;
use std::str;
use std::error::Error;
use std::collections::HashMap;

struct BackServer {
    subdomain: String,
    upstream: SocketAddr,
}

static SERVERS: Lazy<HashMap<String, BackServer>> = Lazy::new(|| {
    let mut servers: HashMap<String, BackServer> = HashMap::new();
    {
        let server = BackServer {
            subdomain: "service1".to_string(),
            upstream: "127.0.0.1:10001".parse().unwrap()
        };
        servers.insert(server.subdomain.clone(), server);
    }
    {
        let server = BackServer {
            subdomain: "service2".to_string(),
            upstream: "127.0.0.1:10002".parse().unwrap()
        };
        servers.insert(server.subdomain.clone(), server);
    }
    servers
});

#[derive(Debug)]
struct Payload {
    subdomain: String,
    payload: Bytes,
}
fn parse_payload(buf: &[u8]) -> Payload {
    let mut bytes = Bytes::copy_from_slice(buf);
    let len = bytes.get_u32() as usize;
    let subdomain = bytes.copy_to_bytes(len);
    let subdomain = str::from_utf8(&subdomain).expect("failed to parse string").to_string();
    let len = bytes.get_u32() as usize;
    let payload = bytes.copy_to_bytes(len);

    Payload {
        subdomain,
        payload
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            let mut stdout = io::stdout();

            let n = socket
                .peek(&mut buf)
                .await
                .expect("failed to read data from socket");
            if n == 0 {
                return;
            }
            let Payload { subdomain, payload: _ } = parse_payload(&buf[0..n]);

            let remote_addr = if let Some(server) = SERVERS.get(&subdomain) {
                server.upstream
            } else {
                println!("{} not found in SERVERS", subdomain);
                return;
            };
            let mut backend = TcpStream::connect(remote_addr).await.expect("failed to connect to backend service");

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");
                if n == 0 {
                    return;
                }
                let Payload { subdomain, payload } = parse_payload(&buf[0..n]);

                stdout.write_all(format!("[{}]: ", subdomain).as_bytes()).await.expect("failed to write data to stdout");
                stdout.write_all(&payload).await.expect("failed to write data to stdout");

                backend
                    .write_all(&payload)
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
