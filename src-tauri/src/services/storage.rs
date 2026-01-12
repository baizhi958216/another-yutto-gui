// Storage service - to be implemented
use rusqlite::Connection;

pub struct Storage {
    pub db_path: String,
}

impl Storage {
    pub fn new(db_path: String) -> Result<Self, rusqlite::Error> {
        Ok(Self { db_path })
    }

    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;

        // Create history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                thumbnail TEXT,
                download_date INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                quality TEXT,
                size INTEGER
            )",
            [],
        )?;

        Ok(())
    }
}
