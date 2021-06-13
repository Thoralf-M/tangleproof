use crate::error::{Error, Result};
use rocksdb::{DBCompressionType, Options, DB};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// Key value storage adapter.
pub struct RocksdbStorage {
    db: Arc<Mutex<DB>>,
}

impl RocksdbStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut opts = Options::default();
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open(&opts, path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub(crate) async fn get(&self, key: &str) -> Result<String> {
        match self.db.lock().await.get(key.as_bytes()) {
            Ok(Some(r)) => Ok(String::from_utf8_lossy(&r).to_string()),
            Ok(None) => Err(Error::RecordNotFound),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) async fn set(&mut self, key: &str, record: String) -> Result<()> {
        self.db
            .lock()
            .await
            .put(key.as_bytes(), record.as_bytes())?;
        Ok(())
    }
}
