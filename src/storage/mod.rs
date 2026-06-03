
use sled::Db;

pub struct Storage {
    pub db: Db,
}

impl Storage {
    pub fn open(path: &str) -> Self {
        Self { db: sled::open(path).expect("storage") }
    }
}
