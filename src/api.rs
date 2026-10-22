use std::{
  fs, io,
  net::SocketAddr,
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};

use axum::{
  extract::{ConnectInfo, Path as AxumPath, RawQuery, State},
  http::{HeaderMap, StatusCode},
  Json,
};
use serde::{Deserialize, Serialize};

use crate::{
  context_menu::{self, complete_settings, enabled_options, MenuOption, MenuSetting},
  ignore_rules::{normalize_patterns, validate_pattern},
  raw::form_values,
  search,
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
  #[serde(default = "primary_by_default")]
  pub primary: bool,
}

fn primary_by_default() -> bool {
  true
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
pub struct MenuOptionDto {
  id: &'static str,
  label: &'static str,
  single_label: &'static str,
  description: &'static str,
  enabled: bool,
}

#[derive(Deserialize)]
pub struct MenuOptionUpdate {
  id: String,
  enabled: bool,
}

#[derive(Serialize)]
pub struct SettingsResponse {
  context_menu: SettingState,
  context_menu_options: Vec<MenuOptionDto>,
  run_on_startup: SettingState,
  url_base: String,
  ignore_patterns: Vec<String>,
  respect_gitignore: bool,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
  context_menu: Option<bool>,
  context_menu_options: Option<Vec<MenuOptionUpdate>>,
  run_on_startup: Option<bool>,
  url_base: Option<String>,
  ignore_patterns: Option<Vec<String>>,
  respect_gitignore: Option<bool>,
}

#[derive(Serialize)]
pub struct DashboardItemDto {
  id: String,
  kind: &'static str,
  paths: Vec<String>,
  share_url: String,
  direct_url: String,
  shared_at: Option<i64>,
}

#[derive(Serialize)]
pub struct DashboardResponse {
  items: Vec<DashboardItemDto>,
}

#[derive(Serialize)]
pub struct DirectoryEntryDto {
  name: String,
  is_dir: bool,
  size: Option<u64>,
  modified: Option<i64>,
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
  file_size: Option<u64>,
  modified: Option<i64>,
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

pub(crate) fn modified_millis(metadata: &fs::Metadata) -> Option<i64> {
  let modified = metadata.modified().ok()?;

  Some(match modified.duration_since(UNIX_EPOCH) {
    Ok(since_epoch) => since_epoch.as_millis() as i64,
    Err(before_epoch) => -(before_epoch.duration().as_millis() as i64),
  })
}

fn dashboard_item(url_base: &str, item: &HostedItem) -> DashboardItemDto {
  DashboardItemDto {
    id: item.id.clone(),
    kind: item.source.kind(),
    paths: item.source.paths().iter().map(|path| path.to_string_lossy().to_string()).collect(),
    share_url: build_share_url(url_base, item, false),
    direct_url: build_share_url(url_base, item, true),
    shared_at: item.shared_at,
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

#[derive(Serialize)]
pub struct SearchResponse {
  entries: Vec<DirectoryEntryDto>,
  truncated: bool,
}

pub async fn api_search_root_handler(
  AxumPath(host_id): AxumPath<String>,
  RawQuery(query): RawQuery,
  State(state): State<AppState>,
) -> Result<Json<SearchResponse>, StatusCode> {
  run_search(host_id, String::new(), query, state).await
}

pub async fn api_search_subpath_handler(
  AxumPath((host_id, subpath)): AxumPath<(String, String)>,
  RawQuery(query): RawQuery,
  State(state): State<AppState>,
) -> Result<Json<SearchResponse>, StatusCode> {
  run_search(host_id, subpath, query, state).await
}

async fn run_search(
  host_id: String,
  subpath: String,
  query: Option<String>,
  state: AppState,
) -> Result<Json<SearchResponse>, StatusCode> {
  let needle = form_values(query.unwrap_or_default().as_bytes(), "q")
    .into_iter()
    .next()
    .unwrap_or_default();

  let item = state
    .db
    .item(&host_id)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  if needle.trim().is_empty() {
    return Ok(Json(SearchResponse { entries: Vec::new(), truncated: false }));
  }

  let rules = state.ignore_rules();
  let allow_external = state.config.allow_external_symlinks;

  let outcome = tokio::task::spawn_blocking(move || search::search(&item, &subpath, &needle, allow_external, &rules))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  Ok(Json(SearchResponse {
    entries: outcome
      .found
      .into_iter()
      .map(|found| DirectoryEntryDto {
        name: found.name,
        is_dir: found.is_dir,
        size: found.size,
        modified: found.modified,
        subpath: found.subpath,
      })
      .collect(),
    truncated: outcome.truncated,
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
  let rules = state.ignore_rules();
  let resolved = resolve_target(&item, &subpath, allow_external, &rules).ok_or(StatusCode::NOT_FOUND)?;

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
    file_size: None,
    modified: None,
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
          size: metadata.is_file().then_some(metadata.len()),
          modified: modified_millis(&metadata),
          subpath: name.clone(),
          name,
        });
      }

      response.entries = Some(entries);
    }
    Resolved::Path { path, root, relative } if path.is_dir() => {
      let entries_dir = fs::read_dir(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
      let scope = rules.scope_for(&root, &relative);
      let mut entries = Vec::new();

      for entry in entries_dir.flatten() {
        let Some(resolved) = resolve_entry(&root, &entry, allow_external) else {
          continue;
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let is_entry_dir = resolved.metadata.is_dir();
        if scope.is_ignored(&entry.file_name(), is_entry_dir) {
          continue;
        }

        let rel_sub = if response.subpath.is_empty() {
          name.clone()
        } else {
          format!("{}/{}", response.subpath, name)
        };

        entries.push(DirectoryEntryDto {
          name,
          is_dir: is_entry_dir,
          size: (!is_entry_dir).then_some(resolved.metadata.len()),
          modified: modified_millis(&resolved.metadata),
          subpath: rel_sub,
        });
      }

      response.entries = Some(entries);
    }
    Resolved::Path { path, .. } => {
      response.is_dir = false;
      response.zip_url = None;
      if let Ok(metadata) = fs::metadata(&path) {
        response.file_size = Some(metadata.len());
        response.modified = modified_millis(&metadata);
      }

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
    primary: true,
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

pub async fn api_reveal_handler(
  AxumPath(host_id): AxumPath<String>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
  require_host(&headers, &addr, &state)?;

  let item = state
    .db
    .item(&host_id)
    .map_err(item_error)?
    .ok_or_else(|| item_error(ItemError::NotFound))?;

  let path = match &item.source {
    ItemSource::File(path) | ItemSource::Directory(path) => path,
    ItemSource::Collection(_) => {
      return Err((StatusCode::BAD_REQUEST, "Items made of several paths can't be revealed.".to_string()));
    }
  };

  if !path.exists() {
    return Err((StatusCode::NOT_FOUND, "That path doesn't exist anymore.".to_string()));
  }

  crate::reveal::reveal(path)
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("Could not open the file manager: {error}")))?;

  Ok(StatusCode::NO_CONTENT)
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

fn parse_menu_options(updates: &[MenuOptionUpdate]) -> Result<Vec<MenuSetting>, String> {
  let mut settings = Vec::new();
  for update in updates {
    let option = MenuOption::from_id(&update.id).ok_or_else(|| format!("'{}' isn't an option of the menu.", update.id))?;
    settings.push(MenuSetting { option, enabled: update.enabled });
  }

  let settings = complete_settings(settings);
  if enabled_options(&settings).is_empty() {
    return Err("At least one option has to be turned on, or there would be no menu.".to_string());
  }
  Ok(settings)
}

fn current_settings(state: &AppState) -> SettingsResponse {
  let supported = cfg!(windows);
  let menu_options = state.db.context_menu_options();

  SettingsResponse {
    context_menu: SettingState { supported, enabled: context_menu::is_registered(&menu_options) },
    context_menu_options: menu_options
      .iter()
      .map(|setting| MenuOptionDto {
        id: setting.option.id(),
        label: setting.option.submenu_label(),
        single_label: setting.option.single_label(),
        description: setting.option.description(),
        enabled: setting.enabled,
      })
      .collect(),
    run_on_startup: SettingState { supported, enabled: startup::is_enabled() },
    url_base: state.db.url_base(),
    ignore_patterns: state.db.ignore_patterns(),
    respect_gitignore: state.db.respect_gitignore(),
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

  let ignore_patterns = update.ignore_patterns.as_deref().map(normalize_patterns);
  for pattern in ignore_patterns.iter().flatten() {
    validate_pattern(pattern)
      .map_err(|message| (StatusCode::BAD_REQUEST, format!("'{pattern}' isn't a valid pattern: {message}")))?;
  }

  let menu_options = update
    .context_menu_options
    .as_deref()
    .map(parse_menu_options)
    .transpose()
    .map_err(|message| (StatusCode::BAD_REQUEST, message))?;

  if let Some(options) = &menu_options {
    let was_registered = context_menu::is_registered(&state.db.context_menu_options());
    state.db.set_context_menu_options(options).map_err(item_error)?;

    if was_registered && update.context_menu != Some(false) {
      context_menu::register(options).map_err(settings_error)?;
    }
  }

  if let Some(enabled) = update.context_menu {
    let result = if enabled {
      context_menu::register(&state.db.context_menu_options())
    } else {
      context_menu::unregister()
    };
    result.map_err(settings_error)?;
  }

  if let Some(enabled) = update.run_on_startup {
    let result = if enabled { startup::enable() } else { startup::disable() };
    result.map_err(settings_error)?;
  }

  if let Some(url_base) = url_base {
    state.db.set_url_base(&url_base).map_err(item_error)?;
  }

  if let Some(patterns) = ignore_patterns {
    state.db.set_ignore_patterns(&patterns).map_err(item_error)?;
  }

  if let Some(enabled) = update.respect_gitignore {
    state.db.set_respect_gitignore(enabled).map_err(item_error)?;
  }

  state.reload_ignore_rules();

  Ok(Json(current_settings(&state)))
}
