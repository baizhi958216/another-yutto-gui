// Storage service - to be implemented
use rusqlite::Connection;
use std::error::Error;

pub struct Storage {
    pub db_path: String,
}

impl Storage {
    pub fn new(db_path: String) -> Result<Self, rusqlite::Error> {
        Ok(Self { db_path })
    }

    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;

        // Create history table with new schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                thumbnail TEXT,
                download_date INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                video_quality INTEGER NOT NULL,
                audio_quality INTEGER NOT NULL,
                video_only INTEGER DEFAULT 0,
                audio_only INTEGER DEFAULT 0,
                size INTEGER DEFAULT 0,
                comment_file_path TEXT
            )",
            [],
        )?;

        // Migrate old data if quality column exists
        let has_old_quality = conn.prepare("SELECT quality FROM history LIMIT 1").is_ok();

        if has_old_quality {
            // Drop old table and recreate with new schema
            conn.execute("DROP TABLE IF EXISTS history_old", [])?;
            conn.execute("ALTER TABLE history RENAME TO history_old", [])?;
            conn.execute(
                "CREATE TABLE history (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    url TEXT NOT NULL,
                    thumbnail TEXT,
                    download_date INTEGER NOT NULL,
                    file_path TEXT NOT NULL,
                    video_quality INTEGER NOT NULL,
                    audio_quality INTEGER NOT NULL,
                    video_only INTEGER DEFAULT 0,
                    audio_only INTEGER DEFAULT 0,
                    size INTEGER DEFAULT 0,
                    comment_file_path TEXT
                )",
                [],
            )?;
            // Note: Old data will be lost, but this is acceptable for a development version
            conn.execute("DROP TABLE history_old", [])?;
        }

        // Add comment_file_path column if it doesn't exist
        let has_comment_field = conn
            .prepare("SELECT comment_file_path FROM history LIMIT 1")
            .is_ok();

        if !has_comment_field {
            eprintln!("[Storage] Adding comment_file_path column to history table");
            conn.execute("ALTER TABLE history ADD COLUMN comment_file_path TEXT", [])?;
        }

        // Create auth table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth (
                id INTEGER PRIMARY KEY,
                sessdata TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                is_valid BOOLEAN DEFAULT 1
            )",
            [],
        )?;

        Ok(())
    }

    // Generate machine-specific encryption key
    fn get_machine_key(&self) -> String {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let username = whoami::username();
        format!("{}{}", hostname, username)
    }

    // XOR encryption/decryption
    fn xor_cipher(&self, data: &str, key: &str) -> String {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len();

        if key_len == 0 {
            return data.to_string();
        }

        let result: Vec<u8> = data
            .as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_len])
            .collect();

        hex::encode(result)
    }

    fn xor_decipher(&self, encrypted_hex: &str, key: &str) -> Result<String, Box<dyn Error>> {
        let encrypted_bytes = hex::decode(encrypted_hex)?;
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len();

        if key_len == 0 {
            return Ok(String::from_utf8(encrypted_bytes)?);
        }

        let result: Vec<u8> = encrypted_bytes
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_len])
            .collect();

        Ok(String::from_utf8(result)?)
    }

    // Save SESSDATA to database (encrypted)
    pub fn save_sessdata(&self, sessdata: String) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        let key = self.get_machine_key();
        let encrypted = self.xor_cipher(&sessdata, &key);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        // Delete existing auth records and insert new one
        conn.execute("DELETE FROM auth", [])?;
        conn.execute(
            "INSERT INTO auth (id, sessdata, created_at, is_valid) VALUES (1, ?1, ?2, 1)",
            [&encrypted, &timestamp.to_string()],
        )?;

        Ok(())
    }

    // Get SESSDATA from database (decrypted)
    pub fn get_sessdata(&self) -> Result<Option<String>, Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        let key = self.get_machine_key();

        let mut stmt = conn.prepare("SELECT sessdata FROM auth WHERE id = 1 AND is_valid = 1")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let encrypted: String = row.get(0)?;
            let decrypted = self.xor_decipher(&encrypted, &key)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }

    // Clear SESSDATA from database
    pub fn clear_sessdata(&self) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM auth", [])?;
        Ok(())
    }

    // Add history entry
    pub fn add_history_entry(
        &self,
        entry: &crate::models::history::HistoryEntry,
    ) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT INTO history (id, title, url, thumbnail, download_date, file_path, video_quality, audio_quality, video_only, audio_only, size, comment_file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                &entry.id,
                &entry.title,
                &entry.url,
                &entry.thumbnail,
                &entry.download_date,
                &entry.file_path,
                &entry.video_quality,
                &entry.audio_quality,
                entry.video_only.unwrap_or(false) as i32,
                entry.audio_only.unwrap_or(false) as i32,
                &entry.size,
                &entry.comment_file_path,
            ],
        )?;
        Ok(())
    }

    // Get history entries with pagination
    pub fn get_history(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<crate::models::history::HistoryEntry>, Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        let offset = page * page_size;

        let mut stmt = conn.prepare(
            "SELECT id, title, url, thumbnail, download_date, file_path, video_quality, audio_quality, video_only, audio_only, size, comment_file_path
             FROM history
             ORDER BY download_date DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let entries = stmt
            .query_map([page_size, offset], |row| {
                Ok(crate::models::history::HistoryEntry {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    thumbnail: row.get(3)?,
                    download_date: row.get(4)?,
                    file_path: row.get(5)?,
                    video_quality: row.get(6)?,
                    audio_quality: row.get(7)?,
                    video_only: {
                        let val: i32 = row.get(8)?;
                        Some(val != 0)
                    },
                    audio_only: {
                        let val: i32 = row.get(9)?;
                        Some(val != 0)
                    },
                    size: row.get(10)?,
                    comment_file_path: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    // Get all history entries
    pub fn get_all_history(
        &self,
    ) -> Result<Vec<crate::models::history::HistoryEntry>, Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT id, title, url, thumbnail, download_date, file_path, video_quality, audio_quality, video_only, audio_only, size, comment_file_path
             FROM history
             ORDER BY download_date DESC"
        )?;

        let entries = stmt
            .query_map([], |row| {
                Ok(crate::models::history::HistoryEntry {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    thumbnail: row.get(3)?,
                    download_date: row.get(4)?,
                    file_path: row.get(5)?,
                    video_quality: row.get(6)?,
                    audio_quality: row.get(7)?,
                    video_only: {
                        let val: i32 = row.get(8)?;
                        Some(val != 0)
                    },
                    audio_only: {
                        let val: i32 = row.get(9)?;
                        Some(val != 0)
                    },
                    size: row.get(10)?,
                    comment_file_path: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    // Delete history entry
    pub fn delete_history_entry(&self, entry_id: &str) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM history WHERE id = ?1", [entry_id])?;
        Ok(())
    }

    // Clear all history
    pub fn clear_history(&self) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }
}
