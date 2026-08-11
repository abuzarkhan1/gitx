use rusqlite::Connection as RusqliteConnection;
use std::path::Path;
use crate::error::Result;

pub struct Connection {
    pub(crate) inner: RusqliteConnection,
}

impl Connection {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut inner = RusqliteConnection::open(path)?;
        crate::migrations::apply_migrations(&mut inner)?;
        Ok(Self { inner })
    }

    pub fn open_in_memory() -> Result<Self> {
        let mut inner = RusqliteConnection::open_in_memory()?;
        crate::migrations::apply_migrations(&mut inner)?;
        Ok(Self { inner })
    }

    pub fn transaction(&mut self) -> Result<rusqlite::Transaction<'_>> {
        Ok(self.inner.transaction()?)
    }
}
