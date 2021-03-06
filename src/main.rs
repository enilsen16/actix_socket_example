use std::str;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{env, io};

use actix_rt::net::UnixStream;
use actix_server::Server;
use actix_service::pipeline_factory;
use bytes::BytesMut;
use futures_util::future::ok;
use log::{error, info};
use tokio::io::{AsyncReadExt};

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix=trace,basic=trace");
    env_logger::init();

    let count = Arc::new(AtomicUsize::new(0));

    // Bind socket address and start worker(s). By default, the server uses the number of available
    // logical CPU cores as the worker count. For this reason, the closure passed to bind needs
    // to return a service *factory*; so it can be created once per worker.
    Server::build()
        .bind_uds("echo", "/tmp/rust-uds.sock", move || {
            let count = Arc::clone(&count);
            let num2 = Arc::clone(&count);

            pipeline_factory(move |mut stream: UnixStream| {
                let count = Arc::clone(&count);

                async move {
                    let num = count.fetch_add(1, Ordering::SeqCst);
                    let num = num + 1;

                    let mut size = 0;
                    let mut buf = BytesMut::new();

                    loop {
                        match stream.read_buf(&mut buf).await {
                            // end of stream; bail from loop
                            Ok(0) => break,

                            // more bytes to process
                            Ok(bytes_read) => {
                                info!("[{}] read {} bytes", num, bytes_read);
                                info!("{:?}", str::from_utf8(&buf[size..]).unwrap());
                                // stream.write_all(&buf[size..]).await.unwrap();
                                size += bytes_read;
                            }

                            // stream error; bail from loop with error
                            Err(err) => {
                                error!("Stream Error: {:?}", err);
                                return Err(());
                            }
                        }
                    }

                    // send data down service pipeline
                    Ok((buf.freeze(), size))
                }
            })
            .map_err(|err| error!("Service Error: {:?}", err))
            .and_then(move |(_, size)| {
                let num = num2.load(Ordering::SeqCst);
                info!("[{}] total bytes read: {}", num, size);
                ok(size)
            })
        })?
        .workers(4)
        .run()
        .await
}
