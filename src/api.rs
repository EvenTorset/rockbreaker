use std::{
  fs, io,
  net::SocketAddr,
  path::{Path, PathBuf},
};

use axum::{
  extract::{ConnectInfo, Path as AxumPath, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use serde::{Deserialize, Serialize};

use crate::{
  context_menu,
  resolve::{resolve_entry, resolve_target, Resolved},
  startup,
  state::{
    build_share_url, encode_path, entry_name, is_valid_id, normalize_url_base, AppState, HostedItem, ItemError,
    ItemSource,
  },
  token::{tokens_match, TOKEN_HEADER},
};

type ApiError = (StatusCode, String);

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
  pub paths: Vec<String>,
  pub direct: bool,
  #[serde(default)]
  pub id: Option<String>,
  #[serde(default)]
  pub selection: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegisterResponse {
  pub host_id: String,
  pub share_url: String,
  pub reused: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StopResponse {
  status: String,
}

#[derive(Deserialize)]
pub struct RenameRequest {
  new_id: String,
}

#[derive(Serialize)]
pub struct RenameResponse {
  id: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
  token: String,
}

#[derive(Serialize)]
pub struct SettingState {
  supported: bool,
  enabled: bool,
}

#[derive(Serialize)]
pub struct SettingsResponse {
  context_menu: SettingState,
  run_on_startup: SettingState,
  url_base: String,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
  context_menu: Option<bool>,
  run_on_startup: Option<bool>,
  url_base: Option<String>,
}

#[derive(Serialize)]
pub struct DashboardItemDto {
  id: String,
  kind: &'static str,
  paths: Vec<String>,
  share_url: String,
  direct_url: String,
}

#[derive(Serialize)]
pub struct DashboardResponse {
  items: Vec<DashboardItemDto>,
}

#[derive(Serialize)]
pub struct DirectoryEntryDto {
  name: String,
  is_dir: bool,
  size: String,
  subpath: String,
}

#[derive(Serialize)]
pub struct ItemDetailResponse {
  id: String,
  kind: &'static str,
  root_name: String,
  item_count: Option<usize>,
  file_name: String,
  is_dir: bool,
  is_host: bool,
  file_size: String,
  subpath: String,
  parent_subpath: Option<String>,
  entries: Option<Vec<DirectoryEntryDto>>,
  zip_url: Option<String>,
  download_url: String,
}

fn item_error(error: ItemError) -> ApiError {
  let status = match error {
    ItemError::InvalidId | ItemError::NoPaths | ItemError::DuplicateName(_) | ItemError::NotUnicode(_) => {
      StatusCode::BAD_REQUEST
    }
    ItemError::IdTaken | ItemError::AlreadyHostedAs(_) => StatusCode::CONFLICT,
    ItemError::NotFound => StatusCode::NOT_FOUND,
    ItemError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
  };

  (status, error.to_string())
}

fn is_host_request(headers: &HeaderMap, addr: &SocketAddr, state: &AppState) -> bool {
  addr.ip().is_loopback()
    && headers
      .get(TOKEN_HEADER)
      .and_then(|value| value.to_str().ok())
      .zip(state.db.token())
      .is_some_and(|(provided, expected)| tokens_match(provided, &expected))
}

fn require_host(headers: &HeaderMap, addr: &SocketAddr, state: &AppState) -> Result<(), ApiError> {
  if is_host_request(headers, addr, state) {
    Ok(())
  } else {
    Err((StatusCode::FORBIDDEN, String::new()))
  }
}

fn format_bytes(bytes: u64) -> String {
  const KB: u64 = 1024;
  const MB: u64 = KB * 1024;
  const GB: u64 = MB * 1024;

  if bytes >= GB {
    format!("{:.2} GB", bytes as f64 / GB as f64)
  } else if bytes >= MB {
    format!("{:.2} MB", bytes as f64 / MB as f64)
  } else if bytes >= KB {
    format!("{:.2} kB", bytes as f64 / KB as f64)
  } else {
    format!("{} B", bytes)
  }
}

fn dashboard_item(url_base: &str, item: &HostedItem) -> DashboardItemDto {
  DashboardItemDto {
    id: item.id.clone(),
    kind: item.source.kind(),
    paths: item.source.paths().iter().map(|path| path.to_string_lossy().to_string()).collect(),
    share_url: build_share_url(url_base, item, false),
    direct_url: build_share_url(url_base, item, true),
  }
}

pub async fn api_dashboard_handler(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<DashboardResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  let url_base = state.db.url_base();
  let items = state.db.items().map_err(item_error)?;

  Ok(Json(DashboardResponse {
    items: items.iter().map(|item| dashboard_item(&url_base, item)).collect(),
  }))
}

pub async fn api_item_root_detail_handler(
  AxumPath(host_id): AxumPath<String>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<ItemDetailResponse>, StatusCode> {
  let is_host = is_host_request(&headers, &addr, &state);
  fetch_item_detail(host_id, String::new(), is_host, state)
}

pub async fn api_item_subpath_detail_handler(
  AxumPath((host_id, subpath)): AxumPath<(String, String)>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<ItemDetailResponse>, StatusCode> {
  let is_host = is_host_request(&headers, &addr, &state);
  fetch_item_detail(host_id, subpath, is_host, state)
}

fn raw_url(id: &str, subpath: &str) -> String {
  if subpath.is_empty() {
    format!("/!/raw/{id}")
  } else {
    format!("/!/raw/{id}/{}", encode_path(subpath))
  }
}

fn fetch_item_detail(
  host_id: String,
  subpath: String,
  is_host: bool,
  state: AppState,
) -> Result<Json<ItemDetailResponse>, StatusCode> {
  let item = state
    .db
    .item(&host_id)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  let allow_external = state.config.allow_external_symlinks;
  let resolved = resolve_target(&item, &subpath, allow_external).ok_or(StatusCode::NOT_FOUND)?;

  let parent_subpath = if !subpath.is_empty() {
    Path::new(&subpath)
      .parent()
      .map(|p| p.to_string_lossy().to_string())
  } else {
    None
  };

  let root_name = item.root_name().unwrap_or_default();
  let viewed_name = Path::new(&subpath)
    .file_name()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or_else(|| root_name.clone());

  let mut response = ItemDetailResponse {
    id: item.id.clone(),
    kind: item.source.kind(),
    root_name,
    item_count: match &item.source {
      ItemSource::Collection(paths) => Some(paths.len()),
      _ => None,
    },
    file_name: viewed_name,
    is_dir: true,
    is_host,
    file_size: "-".to_string(),
    parent_subpath,
    entries: None,
    zip_url: Some(format!("{}?zip=1", raw_url(&item.id, &subpath))),
    download_url: raw_url(&item.id, &subpath),
    subpath,
  };

  match resolved {
    Resolved::CollectionRoot => {
      let mut entries = Vec::new();

      for path in item.source.paths() {
        let Ok(metadata) = fs::metadata(path) else {
          continue;
        };
        if !metadata.is_dir() && !metadata.is_file() {
          continue;
        }

        let name = entry_name(path);
        entries.push(DirectoryEntryDto {
          is_dir: metadata.is_dir(),
          size: if metadata.is_dir() { "-".to_string() } else { format_bytes(metadata.len()) },
          subpath: name.clone(),
          name,
        });
      }

      response.entries = Some(entries);
    }
    Resolved::Path { path, root } if path.is_dir() => {
      let entries_dir = fs::read_dir(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
      let mut entries = Vec::new();

      for entry in entries_dir.flatten() {
        let Some(resolved) = resolve_entry(&root, &entry, allow_external) else {
          continue;
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let is_entry_dir = resolved.metadata.is_dir();

        let rel_sub = if response.subpath.is_empty() {
          name.clone()
        } else {
          format!("{}/{}", response.subpath, name)
        };

        entries.push(DirectoryEntryDto {
          name,
          is_dir: is_entry_dir,
          size: if is_entry_dir { "-".to_string() } else { format_bytes(resolved.metadata.len()) },
          subpath: rel_sub,
        });
      }

      response.entries = Some(entries);
    }
    Resolved::Path { path, .. } => {
      response.is_dir = false;
      response.zip_url = None;
      response.file_size = fs::metadata(&path)
        .map(|m| format_bytes(m.len()))
        .unwrap_or_else(|_| "Unknown size".to_string());

      response.download_url = if response.subpath.is_empty() {
        raw_url(&item.id, &response.file_name)
      } else {
        raw_url(&item.id, &response.subpath)
      };
    }
  }

  Ok(Json(response))
}

pub fn register_item(
  state: &AppState,
  paths: Vec<PathBuf>,
  direct: bool,
  id: Option<&str>,
) -> Result<RegisterResponse, ItemError> {
  let source = ItemSource::from_paths(paths)?;
  let (item, reused) = state.db.register(source, id)?;

  if reused {
    println!("[IPC] Reused active Host ID '{}'", item.id);
  } else {
    println!("[IPC] New {} hosted under ID '{}'", item.source.kind(), item.id);
  }

  Ok(RegisterResponse {
    share_url: build_share_url(&state.db.url_base(), &item, direct),
    host_id: item.id,
    reused,
  })
}

pub async fn api_register_handler(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
  Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  if payload.id.as_deref().is_some_and(|id| !is_valid_id(id)) {
    return Err(item_error(ItemError::InvalidId));
  }

  let mut paths = Vec::new();
  for path in &payload.paths {
    let canonical = dunce::canonicalize(path)
      .map_err(|e| (StatusCode::BAD_REQUEST, format!("Cannot resolve path '{path}': {e}")))?;
    paths.push(canonical);
  }

  let response = if payload.selection {
    state.selections.submit(state.clone(), payload.direct, paths).await
  } else {
    register_item(&state, paths, payload.direct, payload.id.as_deref())
  };

  response.map(Json).map_err(item_error)
}

pub async fn api_stop_handler(
  AxumPath(host_id): AxumPath<String>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<StopResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  let remaining = state.db.remove(&host_id).map_err(item_error)?;
  println!("[HOST] Stopped hosting ID '{}'. Active items remaining: {}", host_id, remaining);

  Ok(Json(StopResponse {
    status: "stopped".to_string(),
  }))
}

pub async fn api_rename_handler(
  AxumPath(host_id): AxumPath<String>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
  Json(payload): Json<RenameRequest>,
) -> Result<Json<RenameResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  state.db.rename(&host_id, &payload.new_id).map_err(item_error)?;
  println!("[HOST] Renamed ID '{}' to '{}'", host_id, payload.new_id);

  Ok(Json(RenameResponse { id: payload.new_id }))
}

pub async fn api_regenerate_token_handler(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<TokenResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  let token = state.db.regenerate_token().map_err(item_error)?;
  println!("[HOST] Generated a new host token");

  Ok(Json(TokenResponse { token }))
}

fn current_settings(state: &AppState) -> SettingsResponse {
  let supported = cfg!(windows);

  SettingsResponse {
    context_menu: SettingState { supported, enabled: context_menu::is_registered() },
    run_on_startup: SettingState { supported, enabled: startup::is_enabled() },
    url_base: state.db.url_base(),
  }
}

pub async fn api_settings_handler(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  Ok(Json(current_settings(&state)))
}

fn settings_error(error: io::Error) -> ApiError {
  let status = if error.kind() == io::ErrorKind::Unsupported {
    StatusCode::NOT_IMPLEMENTED
  } else {
    StatusCode::INTERNAL_SERVER_ERROR
  };

  (status, error.to_string())
}

pub async fn api_update_settings_handler(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
  Json(update): Json<SettingsUpdate>,
) -> Result<Json<SettingsResponse>, ApiError> {
  require_host(&headers, &addr, &state)?;

  let url_base = update
    .url_base
    .as_deref()
    .map(normalize_url_base)
    .transpose()
    .map_err(|message| (StatusCode::BAD_REQUEST, message))?;

  if let Some(enabled) = update.context_menu {
    let result = if enabled { context_menu::register() } else { context_menu::unregister() };
    result.map_err(settings_error)?;
  }

  if let Some(enabled) = update.run_on_startup {
    let result = if enabled { startup::enable() } else { startup::disable() };
    result.map_err(settings_error)?;
  }

  if let Some(url_base) = url_base {
    state.db.set_url_base(&url_base).map_err(item_error)?;
  }

  Ok(Json(current_settings(&state)))
}
