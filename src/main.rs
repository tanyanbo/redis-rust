mod connection;
mod proto;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use connection::handle_connection;
use tokio::{
    self,
    io::{self},
    net::TcpListener,
    spawn,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let fut = handle_connection(stream, Arc::clone(&db));
            spawn(fut);
        }
    }
}
