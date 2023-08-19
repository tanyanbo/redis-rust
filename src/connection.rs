use std::io::ErrorKind;

use crate::proto::{parse, Command, ParserError};
use tokio::{
    self,
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub async fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = [0; 256];
        let result = stream.try_read(&mut buf);
        match result {
            Ok(0) => return Ok(()),
            Ok(bytes) => {
                let command = parse(&buf[..bytes]);
                println!("{:?}", command);
                let response = handle_command(command);
                stream.write(response.as_bytes()).await?;
                stream.flush().await?;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }
}

fn handle_command(command: Result<Command, ParserError>) -> String {
    match command {
        Ok(Command::Array { values }) => {
            if let Some(command) = values.get(0) {
                match command {
                    Command::BulkString { value } if *value == String::from("ping") => {
                        "+PONG\r\n".into()
                    }
                    Command::BulkString { value } if *value == String::from("echo") => {
                        handle_echo(values)
                    }
                    _ => "+Unsupported\r\n".into(),
                }
            } else {
                "Empty command".into()
            }
        }
        Ok(_) => "+Unsupported\r\n".into(),
        Err(e) => {
            let err_string = format!("{:?}", e);
            err_string.into()
        }
    }
}

fn handle_echo(values: Vec<Command>) -> String {
    let command = values.get(1);
    if let Some(command) = command {
        if let Command::BulkString { value } = command {
            return format!("${}\r\n{}\r\n", value.len(), value);
        } else {
            return "+Invalid command\r\n".to_string();
        }
    }
    return "+Invalid command\r\n".to_string();
}
