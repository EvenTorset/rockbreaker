use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Mutex, MutexGuard},
  time::Duration,
};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use crate::{
  context_menu::{default_settings, settings_from_string, settings_to_string, MenuSetting},
  ignore_rules::{normalize_patterns, IgnoreRules},
  state::{generate_host_id, is_valid_id, now_millis, HostedItem, ItemError, ItemSource, DEFAULT_URL_BASE},
  token::generate_host_token,
};

/// Bump this and add a migration step when the schema changes.
const SCHEMA_VERSION: i64 = 2;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

const TOKEN_KEY: &str = "token";
const URL_BASE_KEY: &str = "url_base";
const IGNORE_PATTERNS_KEY: &str = "ignore_patterns";
const RESPECT_GITIGNORE_KEY: &str = "respect_gitignore";
const CONTEXT_MENU_OPTIONS_KEY: &str = "context_menu_options";

type Result<T> = std::result::Result<T, ItemError>;

pub struct Database {
  conn: Mutex<Connection>,
}

impl Database {
  pub fn open(path: &Path) -> std::result::Result<Self, String> {
    Self::open_inner(path).map_err(|e| format!("Could not open the database at {}: {}", path.display(), e))
  }

  fn open_inner(path: &Path) -> rusqlite::Result<Self> {
    if let Some(dir) = path.parent() {
      let _ = fs::create_dir_all(dir);
    }

    let conn = Connection::open(path)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate(&conn)?;

    conn.execute(
      "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
      params![TOKEN_KEY, generate_host_token()],
    )?;

    Ok(Self { conn: Mutex::new(conn) })
  }

  fn lock(&self) -> MutexGuard<'_, Connection> {
    self.conn.lock().unwrap()
  }

  pub fn token(&self) -> Option<String> {
    get_setting(&self.lock(), TOKEN_KEY).ok().flatten()
  }

  pub fn regenerate_token(&self) -> Result<String> {
    let token = generate_host_token();
    self.lock().execute(
      "INSERT INTO settings (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![TOKEN_KEY, token],
    )?;
    Ok(token)
  }

  pub fn url_base(&self) -> String {
    get_setting(&self.lock(), URL_BASE_KEY)
      .ok()
      .flatten()
      .unwrap_or_else(|| DEFAULT_URL_BASE.to_string())
  }

  pub fn set_url_base(&self, url_base: &str) -> Result<()> {
    self.lock().execute(
      "INSERT INTO settings (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![URL_BASE_KEY, url_base],
    )?;
    Ok(())
  }

  pub fn ignore_patterns(&self) -> Vec<String> {
    let text = get_setting(&self.lock(), IGNORE_PATTERNS_KEY).ok().flatten().unwrap_or_default();
    normalize_patterns(&text.lines().collect::<Vec<_>>())
  }

  pub fn set_ignore_patterns(&self, patterns: &[String]) -> Result<()> {
    self.set_setting(IGNORE_PATTERNS_KEY, &patterns.join("\n"))
  }

  pub fn respect_gitignore(&self) -> bool {
    get_setting(&self.lock(), RESPECT_GITIGNORE_KEY)
      .ok()
      .flatten()
      .is_none_or(|value| value != "0")
  }

  pub fn set_respect_gitignore(&self, enabled: bool) -> Result<()> {
    self.set_setting(RESPECT_GITIGNORE_KEY, if enabled { "1" } else { "0" })
  }

  pub fn context_menu_options(&self) -> Vec<MenuSetting> {
    get_setting(&self.lock(), CONTEXT_MENU_OPTIONS_KEY)
      .ok()
      .flatten()
      .map_or_else(default_settings, |text| settings_from_string(&text))
  }

  pub fn set_context_menu_options(&self, settings: &[MenuSetting]) -> Result<()> {
    self.set_setting(CONTEXT_MENU_OPTIONS_KEY, &settings_to_string(settings))
  }

  pub fn ignore_rules(&self) -> IgnoreRules {
    IgnoreRules::new(&self.ignore_patterns(), self.respect_gitignore())
  }

  fn set_setting(&self, key: &str, value: &str) -> Result<()> {
    self.lock().execute(
      "INSERT INTO settings (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![key, value],
    )?;
    Ok(())
  }

  pub fn items(&self) -> Result<Vec<HostedItem>> {
    load_items(&self.lock())
  }

  pub fn item(&self, id: &str) -> Result<Option<HostedItem>> {
    let conn = self.lock();
    let pk: Option<i64> = conn
      .query_row("SELECT pk FROM hosted_items WHERE id = ?1", params![id], |row| row.get(0))
      .optional()?;

    match pk {
      Some(pk) => Ok(Some(load_item(&conn, pk, id.to_string())?)),
      None => Ok(None),
    }
  }

  pub fn register(&self, source: ItemSource, requested_id: Option<&str>) -> Result<(HostedItem, bool)> {
    if requested_id.is_some_and(|id| !is_valid_id(id)) {
      return Err(ItemError::InvalidId);
    }

    let mut conn = self.lock();
    let tx = conn.transaction()?;

    if let Some(existing) = load_items(&tx)?.into_iter().find(|item| item.source == source) {
      return match requested_id {
        Some(id) if id != existing.id => Err(ItemError::AlreadyHostedAs(existing.id)),
        _ => Ok((existing, true)),
      };
    }

    let id = match requested_id {
      Some(id) if id_exists(&tx, id)? => return Err(ItemError::IdTaken),
      Some(id) => id.to_string(),
      None => loop {
        let candidate = generate_host_id();
        if !id_exists(&tx, &candidate)? {
          break candidate;
        }
      },
    };

    let shared_at = now_millis();
    tx.execute(
      "INSERT INTO hosted_items (id, kind, shared_at) VALUES (?1, ?2, ?3)",
      params![id, source.kind(), shared_at],
    )?;
    let pk = tx.last_insert_rowid();
    for (position, path) in source.paths().into_iter().enumerate() {
      tx.execute(
        "INSERT INTO hosted_item_paths (item_pk, position, path) VALUES (?1, ?2, ?3)",
        params![pk, position as i64, path.to_str()],
      )?;
    }
    tx.commit()?;

    Ok((HostedItem { id, source, shared_at: Some(shared_at) }, false))
  }

  pub fn remove(&self, id: &str) -> Result<usize> {
    let conn = self.lock();
    let removed = conn.execute("DELETE FROM hosted_items WHERE id = ?1", params![id])?;
    if removed == 0 {
      return Err(ItemError::NotFound);
    }

    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM hosted_items", [], |row| row.get(0))?;
    Ok(remaining as usize)
  }

  pub fn rename(&self, id: &str, new_id: &str) -> Result<()> {
    if !is_valid_id(new_id) {
      return Err(ItemError::InvalidId);
    }

    let conn = self.lock();
    if !id_exists(&conn, id)? {
      return Err(ItemError::NotFound);
    }
    if id == new_id {
      return Ok(());
    }
    if id_exists(&conn, new_id)? {
      return Err(ItemError::IdTaken);
    }

    conn.execute("UPDATE hosted_items SET id = ?2 WHERE id = ?1", params![id, new_id])?;
    Ok(())
  }
}

pub fn read_token(path: &Path) -> Option<String> {
  let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
  conn.busy_timeout(BUSY_TIMEOUT).ok()?;
  get_setting(&conn, TOKEN_KEY).ok().flatten().filter(|token| !token.is_empty())
}

pub fn read_context_menu_options(path: &Path) -> Vec<MenuSetting> {
  let stored = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok().and_then(|conn| {
    conn.busy_timeout(BUSY_TIMEOUT).ok()?;
    get_setting(&conn, CONTEXT_MENU_OPTIONS_KEY).ok().flatten()
  });

  stored.map_or_else(default_settings, |text| settings_from_string(&text))
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
  let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

  if version > SCHEMA_VERSION {
    return Err(rusqlite::Error::SqliteFailure(
      rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_SCHEMA),
      Some("this database was made by a newer version of Rockbreaker".to_string()),
    ));
  }

  if version < 1 {
    conn.execute_batch(
      "BEGIN;
       CREATE TABLE IF NOT EXISTS settings (
         key   TEXT PRIMARY KEY,
         value TEXT NOT NULL
       );
       CREATE TABLE IF NOT EXISTS hosted_items (
         pk   INTEGER PRIMARY KEY AUTOINCREMENT,
         id   TEXT NOT NULL UNIQUE,
         kind TEXT NOT NULL CHECK (kind IN ('file', 'directory', 'collection'))
       );
       CREATE TABLE IF NOT EXISTS hosted_item_paths (
         item_pk  INTEGER NOT NULL REFERENCES hosted_items (pk) ON DELETE CASCADE,
         position INTEGER NOT NULL,
         path     TEXT NOT NULL,
         PRIMARY KEY (item_pk, position)
       );
       PRAGMA user_version = 1;
       COMMIT;",
    )?;
  }

  if version < 2 {
    // Items shared before this version have no timestamp, so it stays NULL for them rather than pretending.
    conn.execute_batch(
      "BEGIN;
       ALTER TABLE hosted_items ADD COLUMN shared_at INTEGER;
       PRAGMA user_version = 2;
       COMMIT;",
    )?;
  }

  Ok(())
}

fn get_setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
  conn
    .query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| row.get(0))
    .optional()
}

fn id_exists(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
  conn
    .query_row("SELECT 1 FROM hosted_items WHERE id = ?1", params![id], |_| Ok(()))
    .optional()
    .map(|found| found.is_some())
}

fn load_items(conn: &Connection) -> Result<Vec<HostedItem>> {
  let mut statement = conn.prepare("SELECT pk, id FROM hosted_items ORDER BY pk")?;
  let rows: Vec<(i64, String)> = statement
    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
    .collect::<rusqlite::Result<_>>()?;

  rows.into_iter().map(|(pk, id)| load_item(conn, pk, id)).collect()
}

fn load_item(conn: &Connection, pk: i64, id: String) -> Result<HostedItem> {
  let (kind, shared_at): (String, Option<i64>) = conn.query_row(
    "SELECT kind, shared_at FROM hosted_items WHERE pk = ?1",
    params![pk],
    |row| Ok((row.get(0)?, row.get(1)?)),
  )?;

  let mut statement = conn.prepare("SELECT path FROM hosted_item_paths WHERE item_pk = ?1 ORDER BY position")?;
  let mut paths: Vec<PathBuf> = statement
    .query_map(params![pk], |row| row.get::<_, String>(0).map(PathBuf::from))?
    .collect::<rusqlite::Result<_>>()?;

  let source = match (kind.as_str(), paths.len()) {
    ("file", 1) => ItemSource::File(paths.remove(0)),
    ("directory", 1) => ItemSource::Directory(paths.remove(0)),
    ("collection", 2..) => ItemSource::Collection(paths),
    _ => return Err(ItemError::Database(format!("item '{id}' is stored inconsistently"))),
  };

  Ok(HostedItem { id, source, shared_at })
}
