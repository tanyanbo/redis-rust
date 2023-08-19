use std::{
    collections::HashMap,
    io::ErrorKind,
    sync::{Arc, RwLock},
};

use crate::proto::{parse, Command, ParserError};
use tokio::{
    self,
    io::{self, AsyncWriteExt},
    net::TcpStream,
    spawn,
};

pub async fn handle_connection(
    mut stream: TcpStream,
    db: Arc<RwLock<HashMap<String, String>>>,
) -> io::Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = [0; 256];
        let result = stream.try_read(&mut buf);
        match result {
            Ok(0) => return Ok(()),
            Ok(bytes) => {
                let command = parse(&buf[..bytes]);
                println!("{:?}", command);
                let response = handle_command(command, Arc::clone(&db));
                stream.write(response.as_bytes()).await?;
                stream.flush().await?;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }
}

fn handle_command(
    command: Result<Command, ParserError>,
    db: Arc<RwLock<HashMap<String, String>>>,
) -> String {
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
                    Command::BulkString { value } if *value == String::from("set") => {
                        handle_set(Arc::clone(&db), values)
                    }
                    Command::BulkString { value } if *value == String::from("get") => {
                        //
                        handle_get(Arc::clone(&db), values)
                    }
                    _ => "-ERR unsupported command\r\n".into(),
                }
            } else {
                "-ERR empty command".into()
            }
        }
        Ok(_) => "-ERR unsupported command\r\n".into(),
        Err(e) => {
            let err_string = format!("{:?}", e);
            err_string.into()
        }
    }
}

fn handle_set(db: Arc<RwLock<HashMap<String, String>>>, values: Vec<Command>) -> String {
    let mut db = db.write().unwrap();
    let key = values.get(1);
    let value = values.get(2);
    if let (Some(key), Some(value)) = (key, value) {
        if let (Command::BulkString { value: key }, Command::BulkString { value }) = (key, value) {
            db.insert(key.to_string(), value.to_string());
            return "+OK\r\n".into();
        }
    }
    wrong_number_args("set")
}

fn handle_get(db: Arc<RwLock<HashMap<String, String>>>, values: Vec<Command>) -> String {
    // TODO handle db read write failure
    let db = db.read().unwrap();
    let key = values.get(1);
    if let Some(key) = key {
        if let Command::BulkString { value: key } = key {
            let value = db.get(key);
            return if let Some(value) = value {
                get_bulk_string(value)
            } else {
                "_\r\n".into()
            };
        } else {
            return invalid_arg("get");
        }
    }
    wrong_number_args("get")
}

fn handle_echo(values: Vec<Command>) -> String {
    let command = values.get(1);
    if let Some(command) = command {
        if let Command::BulkString { value } = command {
            return get_bulk_string(value);
        } else {
            return invalid_arg("echo");
        }
    }
    wrong_number_args("echo")
}

fn get_bulk_string(value: &String) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}

fn invalid_arg(command: &str) -> String {
    format!("-ERR invalid argument for '{}' command\r\n", command)
}

fn wrong_number_args(command: &str) -> String {
    format!(
        "-ERR wrong number of arguments for '{}' command\r\n",
        command
    )
}
