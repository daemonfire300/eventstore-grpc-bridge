use std::io;
mod rpc;
use rpc::model::Request;
use tokio::{
    io::Interest,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    println!("starting server...");
    let conn = r2d2_sqlite::SqliteConnectionManager::memory();
    let store = eventstore::backend::sqlite::SqliteBackend::new(conn);
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    listener.set_ttl(100).unwrap();
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("new client: {:?}", addr);
                let store = store.clone();
                tokio::spawn(async move {
                    process(store, socket, addr).await;
                });
            }
            Err(e) => {
                println!("accept failed = {:?}", e);
            }
        }
    }
}

/// Basic echo function, input is written back to the client.
async fn process(
    backend: eventstore::backend::sqlite::SqliteBackend,
    socket: TcpStream,
    addr: std::net::SocketAddr,
) {
    let mut buf = [0; 1024];
    loop {
        match socket.ready(Interest::READABLE | Interest::WRITABLE).await {
            Ok(ready) => {
                if ready.is_read_closed() || ready.is_write_closed() {
                    println!("connection closed");
                    return;
                }
            }
            Err(err) => eprintln!("error while checking readiness: {}", err),
        };
        match socket.try_read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }
                println!("received request, trying to decode");
                let raw_req = rmp_serde::from_slice::<Request>(&buf[0..n]);
                match raw_req {
                    Ok(req) => {
                        println!("received request: {:?}", req);
                        if let Ok(agg_vec) = backend.get_aggretate(req.id) {
                            println!("got aggregate with {} versions", agg_vec.len());
                            let version = agg_vec.last().map_or_else(|| 1, |x| x.version + 1);
                            let res = backend.append_event(&eventstore::backend::model::Event {
                                id: req.id,
                                version,
                                data: req.data,
                            });
                            if let Err(err) = res {
                                eprintln!("error while trying to append event: {}", err);
                            };
                        } else {
                            eprintln!("could not find aggregate");
                        };
                    }
                    Err(err) => {
                        eprintln!("error while trying to deserialize request: {}", err)
                    }
                };
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
