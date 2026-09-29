mod protocol;

use bytes::BytesMut;
use protocol::{Frame, encode, parse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Redis server running on 127.0.0.1:6379");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client connected: {:?}", addr);

        tokio::spawn(async move {
            println!("Handling connection from {:?}", addr);

            let mut buffer = BytesMut::with_capacity(4096);

            loop {
                let n = socket.read_buf(&mut buffer).await?;

                if n == 0 {
                    break;
                }

                println!("Received {} bytes", n);
                println!("Buffer: {:?}", &buffer[..]);

                match parse(&mut buffer) {
                    Ok(frame) => {
                        println!("Parsed: {:?}", frame);
                        // Encode the reply and write it to the socket
                        let response = Frame::Simple("PONG".to_string());
                        let mut out = BytesMut::new();
                        encode(&response, &mut out);
                        socket.write_all(&out).await?;
                    }
                    Err(error) => println!("Parse error: {:?}", error),
                }
            }

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
    }
}
