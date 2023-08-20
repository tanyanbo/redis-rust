mod connection;
mod proto;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use chrono::Utc;
use connection::{handle_connection, Db};
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use std::time::Duration;
use tokio::{
    self,
    io::{self},
    net::TcpListener,
    spawn,
    time::sleep,
};

const DELETE_KEYS_SECONDS: u64 = 5;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: Db = Arc::new(RwLock::new(HashMap::new()));

    spawn(delete_expired_keys(Arc::clone(&db)));

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let fut = handle_connection(stream, Arc::clone(&db));
            spawn(fut);
        }
    }
}

async fn delete_expired_keys(db: Db) {
    let mut rng = StdRng::from_entropy();
    loop {
        sleep(Duration::from_secs(DELETE_KEYS_SECONDS)).await;
        let mut db = db.write().unwrap();
        loop {
            let vec = db
                .iter()
                .filter(|(_, value)| value.1.is_some())
                .choose_multiple(&mut rng, 20);
            let expired_keys = vec
                .iter()
                .filter(|(_, value)| Utc::now() > value.1.unwrap())
                .map(|(key, _)| (*key).clone())
                .collect::<Vec<_>>();
            let should_delete_again = expired_keys.len() * 4 > vec.len();
            for key in expired_keys {
                db.remove(&key);
            }

            println!("{:?}", db);
            if !should_delete_again {
                break;
            }
        }
    }
}
