mod proto;

use std::io::ErrorKind;

use proto::{parse, Command};
use tokio::{
    self,
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let fut = handle_connection(stream);
            spawn(fut);
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    loop {
        stream.readable().await?;

        let mut buf = [0; 256];
        let result = stream.try_read(&mut buf);
        match result {
            Ok(0) => return Ok(()),
            Ok(bytes) => {
                let command = parse(&buf[..bytes]);
                println!("{:?}", command);
                match command {
                    Ok(Command::Array { values }) => {
                        if let Some(command) = values.get(0) {
                            match command {
                                Command::BulkString { value } if *value == String::from("ping") => {
                                    stream.write(b"+PONG\r\n").await?;
                                    stream.flush().await?;
                                }
                                _ => {
                                    stream.write(b"+Unsupported\r\n").await?;
                                    stream.flush().await?;
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        stream.write(b"+Unsupported\r\n").await?;
                        stream.flush().await?;
                    }
                    Err(e) => {
                        // let err_string = format!("{:?}", e);
                        // stream.write(err_string.as_bytes()).await?;
                        // stream.flush().await?;
                    }
                };
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }
}
