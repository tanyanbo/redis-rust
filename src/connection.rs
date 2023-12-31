use std::{
    collections::HashMap,
    io::ErrorKind,
    sync::{Arc, RwLock},
};

use crate::proto::{parse, Command, ParserError};
use anyhow::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use tokio::{
    self,
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub type Db = Arc<RwLock<HashMap<String, (String, Option<DateTime<Utc>>)>>>;

pub async fn handle_connection(mut stream: TcpStream, db: Db) -> io::Result<()> {
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

fn handle_command(command: Result<Command, ParserError>, db: Db) -> String {
    match command {
        Ok(Command::Array { values }) => {
            if let Some(command) = values.get(0) {
                match command {
                    Command::BulkString { value }
                        if *value.to_lowercase() == String::from("ping") =>
                    {
                        "+PONG\r\n".into()
                    }
                    Command::BulkString { value }
                        if *value.to_lowercase() == String::from("echo") =>
                    {
                        handle_echo(values)
                    }
                    Command::BulkString { value }
                        if *value.to_lowercase() == String::from("set") =>
                    {
                        handle_set(Arc::clone(&db), values)
                    }
                    Command::BulkString { value }
                        if *value.to_lowercase() == String::from("get") =>
                    {
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

fn handle_set(db: Db, values: Vec<Command>) -> String {
    if values.len() < 3 {
        return wrong_number_args("set");
    }

    let key = values
        .get(1)
        .expect("already checked for number of arguments");
    let value = values
        .get(2)
        .expect("already checked for number of arguments");

    let options = parse_set_options(&values);
    if options.is_err() {
        return invalid_arg("set");
    }

    if let (Command::BulkString { value: key }, Command::BulkString { value }) = (key, value) {
        exectute_set(db, key, value, options.unwrap())
    } else {
        invalid_arg("set")
    }
}

fn exectute_set(
    db: Db,
    key: &String,
    value: &String,
    options: (Option<DateTime<Utc>>, bool, bool, bool),
) -> String {
    let key = key.to_string();
    let value = value.to_string();
    let (expiry, nx, xx, get) = options;
    let mut db = db.write().unwrap();

    let entry = db.get(&key);
    if (entry.is_some() && nx) || (entry.is_none() && xx) || (!nx && !xx) {
        let result = db.insert(key, (value, expiry));
        if get {
            if let Some((prev, _)) = result {
                get_bulk_string(&prev)
            } else {
                get_null_string()
            }
        } else {
            get_simple_string("OK")
        }
    } else {
        get_null_string()
    }
}

fn parse_set_options(values: &Vec<Command>) -> Result<(Option<DateTime<Utc>>, bool, bool, bool)> {
    let mut expiry = None;
    let mut nx = false;
    let mut xx = false;
    let mut get = false;

    let mut i = 3;
    while i < values.len() {
        match &values[i] {
            Command::BulkString { value: arg }
                if *arg.to_lowercase() == String::from("px")
                    || *arg.to_lowercase() == String::from("ex") =>
            {
                let time_arg = values.get(i + 1).ok_or(Error::msg("Invalid arguments"))?;
                if let Command::BulkString { value } = time_arg {
                    let time = value.parse::<usize>()?;
                    if *arg.to_lowercase() == String::from("px") {
                        expiry = Some(Utc::now() + Duration::milliseconds(time as i64));
                    } else {
                        expiry = Some(Utc::now() + Duration::seconds(time as i64));
                    }
                }
                i += 1;
            }
            Command::BulkString { value: arg } if *arg.to_lowercase() == String::from("xx") => {
                xx = true;
            }
            Command::BulkString { value: arg } if *arg.to_lowercase() == String::from("nx") => {
                nx = true;
            }
            Command::BulkString { value: arg } if *arg.to_lowercase() == String::from("get") => {
                get = true;
            }
            _ => {}
        }
        i += 1;
    }

    Ok((expiry, nx, xx, get))
}

fn handle_get(db: Db, values: Vec<Command>) -> String {
    // TODO handle db read write failure
    let key = values.get(1);
    if let Some(key) = key {
        if let Command::BulkString { value: key } = key {
            let read_db = db.read().unwrap();
            let value = read_db.get(key);
            return if let Some((value, expiry)) = value {
                if expiry.is_none() || Utc::now() < (*expiry).unwrap() {
                    get_bulk_string(value)
                } else {
                    drop(read_db);
                    let mut write_db = db.write().unwrap();
                    write_db.remove(key);
                    get_null_string()
                }
            } else {
                get_null_string()
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

fn get_bulk_string(value: &str) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}

fn get_null_string() -> String {
    "_\r\n".into()
}

fn get_simple_string(value: &str) -> String {
    format!("+{}\r\n", value)
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
