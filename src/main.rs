use std::io;

use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    println!("starting server...");
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("new client: {:?}", addr);
                tokio::spawn(async move {
                    process(socket, addr).await;
                });
            }
            Err(e) => {
                println!("accept failed = {:?}", e);
            }
        }
    }
}

/// Basic echo function, input is written back to the client.
async fn process(socket: TcpStream, addr: std::net::SocketAddr) {
    let mut buf = [0; 1024];
    loop {
        match socket.try_read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }
                match socket.try_write(&buf[0..n]) {
                    Ok(n) => println!("written {} bytes", n),
                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(err) => {
                        eprintln!("error while trying to read (addr: {}): {}", addr, err)
                    }
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => continue,
            Err(err) => {
                eprintln!("error while trying to read (addr: {}): {}", addr, err)
            }
        }
    }
}
