use std::{
  fmt,
  path::{Path, PathBuf},
  sync::Arc,
};

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use rand::{distributions::Alphanumeric, Rng};

use crate::{config::Config, db::Database, selection::SelectionBatcher};

pub const DEFAULT_URL_BASE: &str = "http://localhost";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemSource {
  File(PathBuf),
  Directory(PathBuf),
  Collection(Vec<PathBuf>),
}

impl ItemSource {
  pub fn from_paths(mut paths: Vec<PathBuf>) -> Result<Self, ItemError> {
    if paths.is_empty() {
      return Err(ItemError::NoPaths);
    }
    if let Some(path) = paths.iter().find(|path| path.to_str().is_none()) {
      return Err(ItemError::NotUnicode(path.clone()));
    }

    paths.sort_by_cached_key(|path| (entry_name(path).to_lowercase(), path.clone()));
    paths.dedup();

    if let [path] = paths.as_slice() {
      return Ok(if path.is_dir() { Self::Directory(paths.remove(0)) } else { Self::File(paths.remove(0)) });
    }

    for pair in paths.windows(2) {
      if entry_name(&pair[0]) == entry_name(&pair[1]) {
        return Err(ItemError::DuplicateName(entry_name(&pair[0])));
      }
    }
    Ok(Self::Collection(paths))
  }

  pub fn kind(&self) -> &'static str {
    match self {
      Self::File(_) => "file",
      Self::Directory(_) => "directory",
      Self::Collection(_) => "collection",
    }
  }

  pub fn paths(&self) -> Vec<&Path> {
    match self {
      Self::File(path) | Self::Directory(path) => vec![path.as_path()],
      Self::Collection(paths) => paths.iter().map(PathBuf::as_path).collect(),
    }
  }
}

/// What an entry is called when shared. Drive roots and the like have no file name.
pub fn entry_name(path: &Path) -> String {
  path
    .file_name()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or_else(|| "shared_item".to_string())
}

#[derive(Debug, Clone)]
pub struct HostedItem {
  pub id: String,
  pub source: ItemSource,
}

impl HostedItem {
  /// The name of a shared file or folder. A collection doesn't have one.
  pub fn root_name(&self) -> Option<String> {
    match &self.source {
      ItemSource::File(path) | ItemSource::Directory(path) => Some(entry_name(path)),
      ItemSource::Collection(_) => None,
    }
  }
}

#[derive(Debug, Clone)]
pub enum ItemError {
  InvalidId,
  IdTaken,
  AlreadyHostedAs(String),
  NotFound,
  NoPaths,
  DuplicateName(String),
  NotUnicode(PathBuf),
  Database(String),
}

impl fmt::Display for ItemError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidId => write!(f, "An ID can only contain letters, numbers, underscores and dashes."),
      Self::IdTaken => write!(f, "That ID is already in use."),
      Self::AlreadyHostedAs(id) => {
        write!(f, "That item is already shared as '{id}'. Rename it from the dashboard to change its ID.")
      }
      Self::NotFound => write!(f, "Hosted item not found."),
      Self::NoPaths => write!(f, "There is nothing to share."),
      Self::DuplicateName(name) => write!(f, "Can't share several items that are all named '{name}' together."),
      Self::NotUnicode(path) => write!(f, "Can't share '{}' because its path isn't valid Unicode.", path.display()),
      Self::Database(message) => write!(f, "Database error: {message}"),
    }
  }
}

impl From<rusqlite::Error> for ItemError {
  fn from(error: rusqlite::Error) -> Self {
    Self::Database(error.to_string())
  }
}

pub fn is_valid_id(id: &str) -> bool {
  !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn generate_host_id() -> String {
  rand::thread_rng()
    .sample_iter(Alphanumeric)
    .take(8)
    .map(char::from)
    .collect()
}

pub fn normalize_url_base(input: &str) -> Result<String, String> {
  let url = input.trim().trim_end_matches('/');

  let host = url
    .strip_prefix("https://")
    .or_else(|| url.strip_prefix("http://"))
    .ok_or("The address has to start with http:// or https://")?;

  if host.is_empty() || url.chars().any(char::is_whitespace) {
    return Err("That doesn't look like a valid address.".to_string());
  }
  Ok(url.to_string())
}

const URL_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

pub fn encode_path(path: &str) -> String {
  path
    .split('/')
    .map(|segment| utf8_percent_encode(segment, URL_SEGMENT).to_string())
    .collect::<Vec<_>>()
    .join("/")
}

pub fn build_share_url(url_base: &str, item: &HostedItem, direct: bool) -> String {
  let base = url_base.trim_end_matches('/');

  if !direct {
    return format!("{}/{}", base, item.id);
  }

  match &item.source {
    ItemSource::File(path) => format!("{}/!/raw/{}/{}", base, item.id, encode_path(&entry_name(path))),
    ItemSource::Directory(_) | ItemSource::Collection(_) => format!("{}/!/raw/{}?zip=1", base, item.id),
  }
}

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub db: Arc<Database>,
  pub selections: Arc<SelectionBatcher>,
}
