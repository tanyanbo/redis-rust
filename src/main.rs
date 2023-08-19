mod proto;

use std::io::ErrorKind;

use proto::parse;
use tokio::{
    self,
    io::{self},
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
                println!("{:?}", String::from_utf8_lossy(&buf[..bytes]));
                println!("{:?}", command);
                // match commands.get(0) {
                //     Some(&"ping") => {
                //         stream.write(b"+pong\r\n").await?;
                //         stream.flush().await?;
                //     }
                //     Some(&"echo") => {}
                //     Some(_) | None => {
                //         stream.write(b"+\r\n").await?;
                //     }
                // };
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }
}
