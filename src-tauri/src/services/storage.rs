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
}
