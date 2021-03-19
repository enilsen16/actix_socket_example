use actix_rt::net::{UnixListener, UnixStream};
use std::fs;
use std::path::Path;
use tokio::io::AsyncReadExt;

async fn handle_client(mut stream: UnixStream) {
    let mut read = [0; 1028];

    match stream.read(&mut read).await {
        Ok(0) => {
            println!("Hi");
        }
        Ok(n) => {
            println!("{:?}", std::str::from_utf8(&read[0..n]).unwrap());
        }
        Err(err) => {}
    }
}

async fn setup_socket() {
    static PATH: &str = "/tmp/rust-uds.sock";

    if Path::new(PATH).exists() {
        fs::remove_file(PATH).unwrap_or_default();
    };

    let listener = UnixListener::bind(PATH).unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                handle_client(stream).await;
            }
            Err(_e) => { /* connection failed */ }
        }
    }
}

#[actix_rt::main]
async fn main() {
    setup_socket().await;
}
