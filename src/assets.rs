use axum::{
  body::Body,
  http::{header, HeaderValue, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

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

pub async fn spa_fallback_handler(uri: Uri) -> Response {
  let path = uri.path();

  if let Some(asset) = path.strip_prefix(WEB_PREFIX) {
    return asset_response(asset).unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not found.").into_response());
  }

  if path == "/!" || path.starts_with("/!/") {
    return (StatusCode::NOT_FOUND, "Not found.").into_response();
  }

  if let Some(mut response) = asset_response("index.html") {
    response
      .headers_mut()
      .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    return response;
  }

  (StatusCode::NOT_FOUND, "Frontend assets not found.").into_response()
}
