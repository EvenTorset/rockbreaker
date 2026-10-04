use std::{fs, io::SeekFrom, path::Path, time::UNIX_EPOCH};

use axum::{
  body::Body,
  extract::{Path as AxumPath, State},
  http::{header, HeaderMap, HeaderValue, StatusCode},
  response::{IntoResponse, Response},
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{
  archive::{stream_collection_zip, stream_directory_zip},
  resolve::{resolve_target, Resolved},
  state::{entry_name, AppState, ItemSource},
};

pub async fn raw_item_root_handler(
  AxumPath(host_id): AxumPath<String>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Response {
  serve_raw_item(host_id, String::new(), headers, state).await
}

pub async fn raw_item_subpath_handler(
  AxumPath((host_id, subpath)): AxumPath<(String, String)>,
  headers: HeaderMap,
  State(state): State<AppState>,
) -> Response {
  serve_raw_item(host_id, subpath, headers, state).await
}

async fn serve_raw_item(host_id: String, subpath: String, headers: HeaderMap, state: AppState) -> Response {
  let item = match state.db.item(&host_id) {
    Ok(Some(item)) => item,
    Ok(None) => return (StatusCode::NOT_FOUND, "Hosted item not found or inactive.").into_response(),
    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Unable to look up the hosted item.").into_response(),
  };

  let subpath = match &item.source {
    ItemSource::File(path) if subpath == entry_name(path) => String::new(),
    _ => subpath,
  };

  let allow_external = state.config.allow_external_symlinks;
  let Some(resolved) = resolve_target(&item, &subpath, allow_external) else {
    return (StatusCode::NOT_FOUND, "File or directory not found.").into_response();
  };

  let zip_headers = |name: &str| {
    [
      (header::CONTENT_TYPE, HeaderValue::from_static("application/zip")),
      (header::CONTENT_DISPOSITION, content_disposition(&format!("{name}.zip"))),
    ]
  };

  match resolved {
    Resolved::CollectionRoot => {
      let entries = item
        .source
        .paths()
        .into_iter()
        .map(|path| (entry_name(path), path.to_path_buf()))
        .collect();

      (zip_headers(&item.id), stream_collection_zip(entries, allow_external)).into_response()
    }
    Resolved::Path { path, root } => {
      let requested_name = Path::new(&subpath)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| item.root_name().unwrap_or_else(|| item.id.clone()));

      if path.is_dir() {
        let body = stream_directory_zip(path, root, allow_external);
        return (zip_headers(&requested_name), body).into_response();
      }

      serve_file(&path, &requested_name, &headers).await
    }
  }
}

fn content_disposition(filename: &str) -> HeaderValue {
  let fallback: String = filename
    .chars()
    .map(|c| if c.is_ascii() && !c.is_ascii_control() && c != '"' && c != '\\' && c != '%' { c } else { '_' })
    .collect();

  let mut encoded = String::new();
  for byte in filename.bytes() {
    if byte.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&byte) {
      encoded.push(byte as char);
    } else {
      encoded.push_str(&format!("%{byte:02X}"));
    }
  }

  HeaderValue::from_str(&format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}"))
    .unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

enum ByteRange {
  Full,
  Partial { start: u64, end: u64 },
  Unsatisfiable,
}

fn parse_range(headers: &HeaderMap, etag: &str, len: u64) -> ByteRange {
  let Some(value) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) else {
    return ByteRange::Full;
  };

  if let Some(if_range) = headers.get(header::IF_RANGE) {
    if if_range.to_str().ok() != Some(etag) {
      return ByteRange::Full;
    }
  }

  let Some(spec) = value.strip_prefix("bytes=") else {
    return ByteRange::Full;
  };
  if spec.contains(',') {
    return ByteRange::Full;
  }
  let Some((first, last)) = spec.split_once('-') else {
    return ByteRange::Full;
  };
  let (first, last) = (first.trim(), last.trim());

  if first.is_empty() {
    let Ok(suffix) = last.parse::<u64>() else {
      return ByteRange::Full;
    };
    if suffix == 0 || len == 0 {
      return ByteRange::Unsatisfiable;
    }
    return ByteRange::Partial { start: len.saturating_sub(suffix), end: len - 1 };
  }

  let Ok(start) = first.parse::<u64>() else {
    return ByteRange::Full;
  };
  let end = if last.is_empty() {
    u64::MAX
  } else {
    let Ok(end) = last.parse::<u64>() else {
      return ByteRange::Full;
    };
    end
  };

  if end < start {
    return ByteRange::Full;
  }
  if start >= len {
    return ByteRange::Unsatisfiable;
  }
  ByteRange::Partial { start, end: end.min(len - 1) }
}

fn etag_for(metadata: &fs::Metadata) -> String {
  let modified = metadata
    .modified()
    .ok()
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .map_or(0, |since_epoch| since_epoch.as_nanos());
  format!("\"{:x}-{:x}\"", metadata.len(), modified)
}

async fn serve_file(file_path: &Path, download_name: &str, request_headers: &HeaderMap) -> Response {
  let mut file = match tokio::fs::File::open(file_path).await {
    Ok(f) => f,
    Err(_) => return (StatusCode::NOT_FOUND, "Unable to open file.").into_response(),
  };
  let Ok(metadata) = file.metadata().await else {
    return (StatusCode::INTERNAL_SERVER_ERROR, "Unable to read file.").into_response();
  };

  let len = metadata.len();
  let etag = etag_for(&metadata);

  let mut headers = HeaderMap::new();
  headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
  headers.insert(header::CONTENT_DISPOSITION, content_disposition(download_name));
  headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
  headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
  if let Ok(value) = HeaderValue::from_str(&etag) {
    headers.insert(header::ETAG, value);
  }

  let (status, count) = match parse_range(request_headers, &etag, len) {
    ByteRange::Full => (StatusCode::OK, len),
    ByteRange::Partial { start, end } => {
      if file.seek(SeekFrom::Start(start)).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Unable to read file.").into_response();
      }
      if let Ok(value) = HeaderValue::from_str(&format!("bytes {start}-{end}/{len}")) {
        headers.insert(header::CONTENT_RANGE, value);
      }
      (StatusCode::PARTIAL_CONTENT, end - start + 1)
    }
    ByteRange::Unsatisfiable => {
      if let Ok(value) = HeaderValue::from_str(&format!("bytes */{len}")) {
        headers.insert(header::CONTENT_RANGE, value);
      }
      return (StatusCode::RANGE_NOT_SATISFIABLE, headers).into_response();
    }
  };

  headers.insert(header::CONTENT_LENGTH, HeaderValue::from(count));

  let stream = tokio_util::io::ReaderStream::with_capacity(file.take(count), 64 * 1024);
  (status, headers, Body::from_stream(stream)).into_response()
}
