use crate::proxy_parser::{ProxyItem, ProxyStatus};
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Database {
    db_path: std::path::PathBuf,
}

impl Database {
    pub fn new(app_dir: &Path) -> Result<Self> {
        let db_path = app_dir.join("mtproxy_desk.db");
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let db = Self { db_path };
        db.init()?;
        Ok(db)
    }

    pub fn get_conn(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn init(&self) -> Result<()> {
        let conn = self.get_conn()?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS proxies (
                id TEXT PRIMARY KEY,
                server TEXT NOT NULL,
                port INTEGER NOT NULL,
                secret TEXT NOT NULL,
                status TEXT NOT NULL,
                ping_ms INTEGER,
                last_check TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                source_url TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_fetch TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_proxies_status ON proxies(status);
            CREATE INDEX IF NOT EXISTS idx_proxies_ping ON proxies(ping_ms);
            CREATE INDEX IF NOT EXISTS idx_proxies_fav ON proxies(is_favorite);
            "#,
        )?;

        // Ensure default dual sources exist (SoliSpirit + mtproto-auto-proxies)
        let default_source_1 =
            "https://raw.githubusercontent.com/SoliSpirit/mtproto/master/all_proxies.txt";
        let default_source_2 =
            "https://raw.githubusercontent.com/moshraheem-sudo/mtproto-auto-proxies/main/proxies.txt";
        conn.execute(
            "INSERT OR IGNORE INTO sources (url, enabled) VALUES (?1, 1)",
            params![default_source_1],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO sources (url, enabled) VALUES (?1, 1)",
            params![default_source_2],
        )?;

        Ok(())
    }

    /// Upsert proxies into database. Preserves existing favorite status, ping, and last check if already present.
    pub fn upsert_proxies(&self, proxies: &[ProxyItem]) -> Result<usize> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;
        let mut inserted_or_updated = 0;

        {
            let mut check_stmt = tx.prepare(
                "SELECT is_favorite, ping_ms, status, last_check FROM proxies WHERE id = ?1",
            )?;
            let mut insert_stmt = tx.prepare(
                r#"
                INSERT INTO proxies (id, server, port, secret, status, ping_ms, last_check, is_favorite, source_url)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(id) DO UPDATE SET
                    server = excluded.server,
                    port = excluded.port,
                    secret = excluded.secret,
                    source_url = excluded.source_url
                "#,
            )?;

            for p in proxies {
                let mut is_fav = p.is_favorite;
                let mut current_ping = p.ping_ms;
                let mut current_status = p.status.as_str().to_string();
                let mut current_last_check = p.last_check.clone();

                let mut rows = check_stmt.query(params![&p.id])?;
                if let Some(row) = rows.next()? {
                    is_fav = row.get::<_, i64>(0)? != 0;
                    if let Ok(Some(ping)) = row.get::<_, Option<u64>>(1) {
                        current_ping = Some(ping);
                    }
                    if let Ok(st) = row.get::<_, String>(2)
                        && st != "Unchecked"
                    {
                        current_status = st;
                    }
                    if let Ok(Some(lc)) = row.get::<_, Option<String>>(3) {
                        current_last_check = Some(lc);
                    }
                }

                insert_stmt.execute(params![
                    &p.id,
                    &p.server,
                    p.port,
                    &p.secret,
                    current_status,
                    current_ping,
                    current_last_check,
                    if is_fav { 1 } else { 0 },
                    &p.source_url,
                ])?;
                inserted_or_updated += 1;
            }
        }

        tx.commit()?;
        Ok(inserted_or_updated)
    }

    pub fn get_all_proxies(&self) -> Result<Vec<ProxyItem>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, server, port, secret, status, ping_ms, last_check, is_favorite, source_url FROM proxies",
        )?;

        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(4)?;
            let is_fav_int: i64 = row.get(7)?;
            Ok(ProxyItem {
                id: row.get(0)?,
                server: row.get(1)?,
                port: row.get(2)?,
                secret: row.get(3)?,
                status: ProxyStatus::parse_status(&status_str),
                ping_ms: row.get(5)?,
                last_check: row.get(6)?,
                is_favorite: is_fav_int != 0,
                source_url: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn update_proxy_status(
        &self,
        id: &str,
        status: &ProxyStatus,
        ping_ms: Option<u64>,
        last_check: &str,
    ) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE proxies SET status = ?1, ping_ms = ?2, last_check = ?3 WHERE id = ?4",
            params![status.as_str(), ping_ms, last_check, id],
        )?;
        Ok(())
    }

    pub fn set_favorite(&self, id: &str, is_favorite: bool) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE proxies SET is_favorite = ?1 WHERE id = ?2",
            params![if is_favorite { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn delete_proxy(&self, id: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM proxies WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all_proxies(&self) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM proxies", [])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_sources(&self) -> Result<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT url FROM sources WHERE enabled = 1")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn add_source(&self, url: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO sources (url, enabled) VALUES (?1, 1)",
            params![url],
        )?;
        Ok(())
    }

    pub fn remove_source(&self, url: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM sources WHERE url = ?1", params![url])?;
        Ok(())
    }
}
