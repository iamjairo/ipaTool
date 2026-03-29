use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Option<i64>,
    pub token: String,
    pub email: String,
    pub region: String,
    pub guid: Option<String>,
    pub cookie_user: Option<String>,
    pub cookies: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub id: Option<i64>,
    pub email: String,
    pub password_encrypted: String,
    pub key_id: String,
    pub iv: String,
    pub auth_tag: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: Option<i64>,
    pub username: String,
    pub password_hash: String,
    pub is_default: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: Option<i64>,
    pub token: String,
    pub username: String,
    pub created_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: Option<i64>,
    pub key_id: String,
    pub key_value: String,
    pub is_current: bool,
    pub created_at: Option<String>,
    pub last_rotation: i64,
    pub next_rotation: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: Option<i64>,
    pub app_name: String,
    pub app_id: String,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub account_email: String,
    pub account_region: Option<String>,
    pub download_date: Option<String>,
    pub status: String,
    pub file_size: Option<i64>,
    pub install_url: Option<String>,
    pub artwork_url: Option<String>,
    pub artist_name: Option<String>,
    pub progress: Option<i64>,
    pub error: Option<String>,
    pub created_at: Option<String>,
}

pub struct NewSubscription<'a> {
    pub app_id: &'a str,
    pub app_name: &'a str,
    pub bundle_id: Option<&'a str>,
    pub account_email: &'a str,
    pub account_region: Option<&'a str>,
    pub artwork_url: Option<&'a str>,
    pub artist_name: Option<&'a str>,
}

pub struct Database {
    connection: std::sync::Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let path = Path::new(db_path);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        let connection = Connection::open(path)?;

        // PRAGMA 语句使用 query_row 而不是 execute
        let _ = connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        });
        let _ = connection.query_row("PRAGMA foreign_keys = ON", [], |row| row.get::<_, i32>(0));

        Self::create_tables(&connection)?;
        Self::migrate_tables(&connection)?;
        Self::seed_default_admin(&connection)?;

        Ok(Database {
            connection: std::sync::Mutex::new(connection),
        })
    }

    fn create_tables(conn: &Connection) -> Result<()> {
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token TEXT UNIQUE NOT NULL,
                email TEXT NOT NULL,
                region TEXT DEFAULT 'US',
                guid TEXT,
                cookie_user TEXT,
                cookies TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS credentials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL,
                password_encrypted TEXT NOT NULL,
                key_id TEXT NOT NULL,
                iv TEXT NOT NULL,
                auth_tag TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS encryption_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id TEXT UNIQUE NOT NULL,
                key_value TEXT NOT NULL,
                is_current BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_rotation INTEGER NOT NULL,
                next_rotation INTEGER NOT NULL
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS download_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL,
                app_id TEXT NOT NULL,
                bundle_id TEXT,
                version TEXT,
                account_email TEXT NOT NULL,
                account_region TEXT,
                download_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT DEFAULT 'completed',
                file_size INTEGER,
                install_url TEXT,
                artwork_url TEXT,
                artist_name TEXT,
                progress INTEGER DEFAULT 0,
                error TEXT,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 5,
                download_speed REAL,
                resume_position INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS app_subscriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id TEXT NOT NULL,
                app_name TEXT NOT NULL,
                bundle_id TEXT,
                account_email TEXT NOT NULL,
                account_region TEXT,
                current_version TEXT,
                artwork_url TEXT,
                artist_name TEXT,
                subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_checked DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(app_id, account_email)
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS batch_download_tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_name TEXT NOT NULL,
                status TEXT DEFAULT 'pending',
                total_count INTEGER DEFAULT 0,
                completed_count INTEGER DEFAULT 0,
                failed_count INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                completed_at DATETIME
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS batch_download_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                app_id TEXT NOT NULL,
                app_name TEXT,
                version TEXT,
                account_email TEXT,
                status TEXT DEFAULT 'pending',
                progress INTEGER DEFAULT 0,
                error TEXT,
                retry_count INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (batch_id) REFERENCES batch_download_tasks(id) ON DELETE CASCADE
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS admin_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                is_default BOOLEAN DEFAULT TRUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token TEXT UNIQUE NOT NULL,
                username TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                expires_at DATETIME NOT NULL,
                FOREIGN KEY (username) REFERENCES admin_users(username) ON DELETE CASCADE
            )
        ",
            [],
        )?;

        Ok(())
    }

    fn migrate_tables(conn: &Connection) -> Result<()> {
        let table_info: Vec<(i32, String, String, bool, i32, bool)> = conn
            .prepare("PRAGMA table_info(accounts)")?
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let has_region = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "region");
        if !has_region {
            let _ = conn.execute(
                "ALTER TABLE accounts ADD COLUMN region TEXT DEFAULT 'US'",
                [],
            );
        }

        let table_info: Vec<(i32, String, String, bool, i32, bool)> = conn
            .prepare("PRAGMA table_info(download_records)")?
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let has_progress = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "progress");
        let has_error = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "error");

        if !has_progress {
            let _ = conn.execute(
                "ALTER TABLE download_records ADD COLUMN progress INTEGER DEFAULT 0",
                [],
            );
        }
        if !has_error {
            let _ = conn.execute("ALTER TABLE download_records ADD COLUMN error TEXT", []);
        }

        let has_retry_count = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "retry_count");
        if !has_retry_count {
            let _ = conn.execute(
                "ALTER TABLE download_records ADD COLUMN retry_count INTEGER DEFAULT 5",
                [],
            );
        }

        let has_max_retries = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "max_retries");
        if !has_max_retries {
            let _ = conn.execute(
                "ALTER TABLE download_records ADD COLUMN max_retries INTEGER DEFAULT 5",
                [],
            );
        }

        let has_download_speed = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "download_speed");
        if !has_download_speed {
            let _ = conn.execute(
                "ALTER TABLE download_records ADD COLUMN download_speed REAL",
                [],
            );
        }

        let has_resume_position = table_info
            .iter()
            .any(|(_, name, _, _, _, _)| name == "resume_position");
        if !has_resume_position {
            let _ = conn.execute(
                "ALTER TABLE download_records ADD COLUMN resume_position INTEGER DEFAULT 0",
                [],
            );
        }

        let _ = conn.execute("DELETE FROM encryption_keys WHERE key_id IS NULL", []);

        Ok(())
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn seed_default_admin(conn: &Connection) -> Result<()> {
        let existing = conn
            .query_row(
                "SELECT id FROM admin_users WHERE username = ?",
                params!["admin"],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        if existing.is_none() {
            conn.execute(
                "INSERT INTO admin_users (username, password_hash, is_default) VALUES (?, ?, ?)",
                params!["admin", Self::hash_password("admin"), true],
            )?;
        }

        Ok(())
    }

    pub fn create_admin_user(
        &self,
        username: &str,
        password_hash: &str,
        is_default: bool,
    ) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO admin_users (username, password_hash, is_default) VALUES (?, ?, ?)",
            params![username, password_hash, is_default],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_admin_user(&self, username: &str) -> Result<Option<AdminUser>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM admin_users WHERE username = ?")?;
        let user = stmt
            .query_row(params![username], |row| {
                Ok(AdminUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .optional()?;
        Ok(user)
    }

    pub fn update_admin_password(
        &self,
        username: &str,
        password_hash: &str,
        is_default: bool,
    ) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE admin_users SET password_hash = ?, is_default = ?, updated_at = CURRENT_TIMESTAMP WHERE username = ?",
            params![password_hash, is_default, username],
        )?;
        Ok(())
    }

    pub fn delete_admin_user(&self, username: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM admin_users WHERE username = ?",
            params![username],
        )?;
        Ok(())
    }

    pub fn create_session(&self, token: &str, username: &str, expires_at: &str) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (token, username, expires_at) VALUES (?, ?, ?)",
            params![token, username, expires_at],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_session(&self, token: &str) -> Result<Option<SessionRecord>> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM sessions WHERE expires_at <= CURRENT_TIMESTAMP",
            [],
        )?;
        let mut stmt = conn
            .prepare("SELECT * FROM sessions WHERE token = ? AND expires_at > CURRENT_TIMESTAMP")?;
        let session = stmt
            .query_row(params![token], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    token: row.get(1)?,
                    username: row.get(2)?,
                    created_at: row.get(3)?,
                    expires_at: row.get(4)?,
                })
            })
            .optional()?;
        Ok(session)
    }

    pub fn delete_session(&self, token: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE token = ?", params![token])?;
        Ok(())
    }

    pub fn delete_sessions_by_username(&self, username: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE username = ?", params![username])?;
        Ok(())
    }

    pub fn cleanup_expired_sessions(&self) -> Result<usize> {
        let conn = self.connection.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM sessions WHERE expires_at <= CURRENT_TIMESTAMP",
            [],
        )?;
        Ok(deleted)
    }

    pub fn get_all_accounts(&self) -> Result<Vec<Account>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM accounts")?;
        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    token: row.get(1)?,
                    email: row.get(2)?,
                    region: row.get(3)?,
                    guid: row.get(4)?,
                    cookie_user: row.get(5)?,
                    cookies: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(accounts)
    }

    pub fn get_account_by_token(&self, token: &str) -> Result<Option<Account>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM accounts WHERE token = ?")?;
        let account = stmt
            .query_row(params![token], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    token: row.get(1)?,
                    email: row.get(2)?,
                    region: row.get(3)?,
                    guid: row.get(4)?,
                    cookie_user: row.get(5)?,
                    cookies: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .optional()?;
        Ok(account)
    }

    pub fn save_account(&self, account: &Account) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO accounts (token, email, region, guid, cookie_user, cookies) 
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                account.token,
                account.email,
                account.region,
                account.guid,
                account.cookie_user,
                account.cookies,
            ],
        )?;
        Ok(())
    }

    pub fn delete_account(&self, token: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM accounts WHERE token = ?", params![token])?;
        Ok(())
    }

    pub fn save_credentials(&self, credentials: &Credentials) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO credentials (email, password_encrypted, key_id, iv, auth_tag) 
             VALUES (?, ?, ?, ?, ?)",
            params![
                credentials.email,
                credentials.password_encrypted,
                credentials.key_id,
                credentials.iv,
                credentials.auth_tag,
            ],
        )?;
        Ok(())
    }

    pub fn delete_credentials(&self, email: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM credentials WHERE email = ?", params![email])?;
        Ok(())
    }

    pub fn get_credentials(&self, email: &str) -> Result<Option<Credentials>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM credentials WHERE email = ?")?;
        let cred = stmt
            .query_row(params![email], |row| {
                Ok(Credentials {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_encrypted: row.get(2)?,
                    key_id: row.get(3)?,
                    iv: row.get(4)?,
                    auth_tag: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .optional()?;
        Ok(cred)
    }

    pub fn get_all_credentials(&self) -> Result<Vec<Credentials>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM credentials")?;
        let creds = stmt
            .query_map([], |row| {
                Ok(Credentials {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_encrypted: row.get(2)?,
                    key_id: row.get(3)?,
                    iv: row.get(4)?,
                    auth_tag: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(creds)
    }

    pub fn save_encryption_key(&self, key: &EncryptionKey) -> Result<()> {
        let conn = self.connection.lock().unwrap();

        if key.is_current {
            conn.execute("UPDATE encryption_keys SET is_current = FALSE", [])?;
        }

        conn.execute(
            "INSERT OR REPLACE INTO encryption_keys (key_id, key_value, is_current, last_rotation, next_rotation) 
             VALUES (?, ?, ?, ?, ?)",
            params![
                key.key_id,
                key.key_value,
                if key.is_current { 1i64 } else { 0i64 },
                key.last_rotation,
                key.next_rotation,
            ],
        )?;
        Ok(())
    }

    pub fn get_current_encryption_key(&self) -> Result<Option<EncryptionKey>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM encryption_keys WHERE is_current = 1")?;
        let key = stmt
            .query_row([], |row| {
                Ok(EncryptionKey {
                    id: row.get(0)?,
                    key_id: row.get(1)?,
                    key_value: row.get(2)?,
                    is_current: row.get(3)?,
                    created_at: row.get(4)?,
                    last_rotation: row.get(5)?,
                    next_rotation: row.get(6)?,
                })
            })
            .optional()?;
        Ok(key)
    }

    pub fn get_all_encryption_keys(&self) -> Result<Vec<EncryptionKey>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM encryption_keys ORDER BY created_at DESC")?;
        let keys = stmt
            .query_map([], |row| {
                Ok(EncryptionKey {
                    id: row.get(0)?,
                    key_id: row.get(1)?,
                    key_value: row.get(2)?,
                    is_current: row.get(3)?,
                    created_at: row.get(4)?,
                    last_rotation: row.get(5)?,
                    next_rotation: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(keys)
    }

    pub fn reset_encryption_keys(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM encryption_keys", [])?;
        Ok(())
    }

    pub fn add_download_record(&self, record: &DownloadRecord) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO download_records 
             (app_name, app_id, bundle_id, version, account_email, account_region, status, file_size, install_url, artwork_url, artist_name, progress, error) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.app_name,
                record.app_id,
                record.bundle_id,
                record.version,
                record.account_email,
                record.account_region,
                record.status,
                record.file_size,
                record.install_url,
                record.artwork_url,
                record.artist_name,
                record.progress,
                record.error,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_download_records(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT * FROM download_records ORDER BY download_date DESC")?;
        let records = stmt
            .query_map([], |row| {
                Ok(DownloadRecord {
                    id: row.get(0)?,
                    app_name: row.get(1)?,
                    app_id: row.get(2)?,
                    bundle_id: row.get(3)?,
                    version: row.get(4)?,
                    account_email: row.get(5)?,
                    account_region: row.get(6)?,
                    download_date: row.get(7)?,
                    status: row.get(8)?,
                    file_size: row.get(9)?,
                    install_url: row.get(10)?,
                    artwork_url: row.get(11)?,
                    artist_name: row.get(12)?,
                    progress: row.get(13)?,
                    error: row.get(14)?,
                    created_at: row.get(15)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(records)
    }

    pub fn delete_download_record(&self, id: i64) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM download_records WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn update_download_record(&self, id: i64, updates: &DownloadRecord) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE download_records SET 
             app_name = ?, app_id = ?, bundle_id = ?, version = ?, 
             account_email = ?, account_region = ?, status = ?, 
             file_size = ?, install_url = ?, artwork_url = ?, 
             artist_name = ?, progress = ?, error = ?
             WHERE id = ?",
            params![
                updates.app_name,
                updates.app_id,
                updates.bundle_id,
                updates.version,
                updates.account_email,
                updates.account_region,
                updates.status,
                updates.file_size,
                updates.install_url,
                updates.artwork_url,
                updates.artist_name,
                updates.progress,
                updates.error,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn clear_all_download_records(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM download_records", [])?;
        Ok(())
    }

    // 订阅相关方法
    pub fn add_subscription(&self, subscription: &NewSubscription<'_>) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO app_subscriptions 
             (app_id, app_name, bundle_id, account_email, account_region, artwork_url, artist_name, last_checked) 
             VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            params![
                subscription.app_id,
                subscription.app_name,
                subscription.bundle_id,
                subscription.account_email,
                subscription.account_region,
                subscription.artwork_url,
                subscription.artist_name,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn remove_subscription(&self, app_id: &str, account_email: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM app_subscriptions WHERE app_id = ? AND account_email = ?",
            params![app_id, account_email],
        )?;
        Ok(())
    }

    pub fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT * FROM app_subscriptions ORDER BY subscribed_at DESC")?;
        let subs = stmt
            .query_map([], |row| {
                Ok(Subscription {
                    id: row.get(0)?,
                    app_id: row.get(1)?,
                    app_name: row.get(2)?,
                    bundle_id: row.get(3)?,
                    account_email: row.get(4)?,
                    account_region: row.get(5)?,
                    current_version: row.get(6)?,
                    artwork_url: row.get(7)?,
                    artist_name: row.get(8)?,
                    subscribed_at: row.get(9)?,
                    last_checked: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(subs)
    }

    pub fn update_subscription_version(
        &self,
        app_id: &str,
        account_email: &str,
        new_version: &str,
    ) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE app_subscriptions SET current_version = ?, last_checked = CURRENT_TIMESTAMP 
             WHERE app_id = ? AND account_email = ?",
            params![new_version, app_id, account_email],
        )?;
        Ok(())
    }

    // 批量下载相关方法
    pub fn create_batch_task(&self, task_name: &str, total_count: i64) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO batch_download_tasks (task_name, total_count) VALUES (?, ?)",
            params![task_name, total_count],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn add_batch_item(
        &self,
        batch_id: i64,
        app_id: &str,
        app_name: Option<&str>,
        version: Option<&str>,
        account_email: &str,
    ) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO batch_download_items (batch_id, app_id, app_name, version, account_email) 
             VALUES (?, ?, ?, ?, ?)",
            params![batch_id, app_id, app_name, version, account_email],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_batch_tasks(&self) -> Result<Vec<BatchDownloadTask>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT * FROM batch_download_tasks ORDER BY created_at DESC")?;
        let tasks = stmt
            .query_map([], |row| {
                Ok(BatchDownloadTask {
                    id: row.get(0)?,
                    task_name: row.get(1)?,
                    status: row.get(2)?,
                    total_count: row.get(3)?,
                    completed_count: row.get(4)?,
                    failed_count: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    completed_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tasks)
    }

    pub fn get_batch_items(&self, batch_id: i64) -> Result<Vec<BatchDownloadItem>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM batch_download_items WHERE batch_id = ?")?;
        let items = stmt
            .query_map(params![batch_id], |row| {
                Ok(BatchDownloadItem {
                    id: row.get(0)?,
                    batch_id: row.get(1)?,
                    app_id: row.get(2)?,
                    app_name: row.get(3)?,
                    version: row.get(4)?,
                    account_email: row.get(5)?,
                    status: row.get(6)?,
                    progress: row.get(7)?,
                    error: row.get(8)?,
                    retry_count: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    pub fn update_batch_item(
        &self,
        item_id: i64,
        status: &str,
        progress: i64,
        error: Option<&str>,
        retry_count: i64,
    ) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE batch_download_items SET status = ?, progress = ?, error = ?, retry_count = ? WHERE id = ?",
            params![status, progress, error, retry_count, item_id],
        )?;
        Ok(())
    }

    pub fn update_batch_task_progress(
        &self,
        batch_id: i64,
        completed_count: i64,
        failed_count: i64,
        status: &str,
    ) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "UPDATE batch_download_tasks SET completed_count = ?, failed_count = ?, status = ?, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?",
            params![completed_count, failed_count, status, batch_id],
        )?;
        Ok(())
    }

    pub fn delete_batch_task(&self, batch_id: i64) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM batch_download_tasks WHERE id = ?",
            params![batch_id],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Option<i64>,
    pub app_id: String,
    pub app_name: String,
    pub bundle_id: Option<String>,
    pub account_email: String,
    pub account_region: Option<String>,
    pub current_version: Option<String>,
    pub artwork_url: Option<String>,
    pub artist_name: Option<String>,
    pub subscribed_at: Option<String>,
    pub last_checked: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDownloadTask {
    pub id: Option<i64>,
    pub task_name: String,
    pub status: String,
    pub total_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDownloadItem {
    pub id: Option<i64>,
    pub batch_id: i64,
    pub app_id: String,
    pub app_name: Option<String>,
    pub version: Option<String>,
    pub account_email: String,
    pub status: String,
    pub progress: i64,
    pub error: Option<String>,
    pub retry_count: i64,
    pub created_at: Option<String>,
}
