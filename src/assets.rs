use axum::{
  body::Body,
  extract::State,
  http::{header, HeaderValue, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

use crate::{embed::add_preview, state::AppState};

const WEB_PREFIX: &str = "/!/web/";

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct EmbeddedAssets;

fn asset_response(path: &str) -> Option<Response> {
  let content = EmbeddedAssets::get(path)?;
  let mime = mime_guess::from_path(path).first_or_octet_stream();

  Some(
    (
      [(header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap())],
      Body::from(content.data),
    )
      .into_response(),
  )
}

pub async fn spa_fallback_handler(State(state): State<AppState>, uri: Uri) -> Response {
  let path = uri.path();

  if let Some(asset) = path.strip_prefix(WEB_PREFIX) {
    return asset_response(asset).unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not found.").into_response());
  }

  if path == "/!" || path.starts_with("/!/") {
    return (StatusCode::NOT_FOUND, "Not found.").into_response();
  }

  if let Some(index) = EmbeddedAssets::get("index.html") {
    let html = String::from_utf8_lossy(&index.data).into_owned();
    let html = add_preview(&state, path, html).await;

    return (
      [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))],
      Body::from(html),
    )
      .into_response();
  }

  (StatusCode::NOT_FOUND, "Frontend assets not found.").into_response()
}
