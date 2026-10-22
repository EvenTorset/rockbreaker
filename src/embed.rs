// Discord Component Embeds: https://docs.discord.com/developers/link-previews/component-embeds

use std::path::Path;

use percent_encoding::percent_decode_str;
use serde_json::{json, Value};

use crate::{
  resolve::{resolve_target, Resolved},
  state::{build_share_url, encode_path, entry_name, is_valid_id, AppState, ItemSource},
};

const MAX_PAYLOAD_BYTES: usize = 3000;
const MAX_MARKDOWN_CHARS: usize = 2048;
const MAX_MARKDOWN_FILE_BYTES: u64 = 16 * 1024;
const COLLECTION_NAMES_CHARS: usize = 120;

const ACCENT_COLOR: u32 = 0x5588ff;

const IMAGE_EXTENSIONS: [&str; 6] = ["png", "gif", "jpg", "jpeg", "webp", "avif"];
const VIDEO_EXTENSIONS: [&str; 3] = ["mp4", "webm", "mov"];
const MARKDOWN_EXTENSIONS: [&str; 5] = ["md", "markdown", "mdown", "mkd", "mkdn"];

#[derive(Debug, Clone, PartialEq)]
pub enum Media {
  Image(String),
  Video(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
  File { size: u64 },
  Folder,
  Collection { names: Vec<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
  pub name: String,
  pub kind: Kind,
  pub page_url: String,
  pub download_url: String,
  pub media: Option<Media>,
  pub markdown: Option<String>,
}

pub fn format_size(bytes: u64) -> String {
  const KB: f64 = 1024.0;
  let size = bytes as f64;

  if size >= KB * KB * KB {
    format!("{:.2} GB", size / (KB * KB * KB))
  } else if size >= KB * KB {
    format!("{:.2} MB", size / (KB * KB))
  } else if size >= KB {
    format!("{:.2} kB", size / KB)
  } else {
    format!("{bytes} B")
  }
}

fn extension(name: &str) -> Option<String> {
  let (stem, extension) = name.rsplit_once('.')?;
  (!stem.is_empty()).then(|| extension.to_ascii_lowercase())
}

fn escape_markdown(text: &str) -> String {
  const SPECIAL: &str = "\\*_~|`<>[]():@#!";

  text
    .chars()
    .flat_map(|c| {
      let escape = SPECIAL.contains(c);
      let c = if c.is_control() { ' ' } else { c };
      escape.then_some('\\').into_iter().chain(std::iter::once(c))
    })
    .collect()
}

fn escape_html(text: &str) -> String {
  let mut escaped = String::with_capacity(text.len());
  for c in text.chars() {
    match c {
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&#39;"),
      c => escaped.push(c),
    }
  }
  escaped
}

fn truncate_chars(text: &str, max: usize) -> String {
  if text.chars().count() <= max {
    return text.to_string();
  }
  let mut shortened: String = text.chars().take(max).collect();
  shortened.push('…');
  shortened
}

impl Preview {
  fn detail_with(&self, escape: fn(&str) -> String) -> String {
    match &self.kind {
      Kind::File { size } => format_size(*size),
      Kind::Folder => "Folder".to_string(),
      Kind::Collection { names } => {
        let list = truncate_chars(&names.join(", "), COLLECTION_NAMES_CHARS);
        format!("{} items: {}", names.len(), escape(&list))
      }
    }
  }

  fn detail(&self) -> String {
    self.detail_with(str::to_string)
  }

  fn components(&self, with_media: bool, with_markdown: bool) -> Vec<Value> {
    let mut components = vec![json!({
      "type": 10,
      "content": format!("## {}\n-# {}", escape_markdown(&self.name), self.detail_with(escape_markdown)),
    })];

    if let Some(media) = self.media.as_ref().filter(|_| with_media) {
      let (Media::Image(url) | Media::Video(url)) = media;
      components.push(json!({ "type": 12, "items": [{ "media": { "url": url } }] }));
    }

    if let Some(markdown) = self.markdown.as_ref().filter(|_| with_markdown) {
      components.push(json!({ "type": 14 }));
      components.push(json!({ "type": 10, "content": markdown }));
    }

    components.push(json!({
      "type": 1,
      "components": [
        { "type": 2, "style": 5, "label": "Open", "url": self.page_url },
        { "type": 2, "style": 5, "label": "Download", "url": self.download_url },
      ],
    }));

    components
  }

  fn payload(&self, with_media: bool, with_markdown: bool) -> String {
    let payload = json!({
      "component": {
        "type": 17,
        "accent_color": ACCENT_COLOR,
        "components": self.components(with_media, with_markdown),
      }
    });

    payload.to_string().replace('<', "\\u003c")
  }

  pub fn component_embed(&self) -> Option<String> {
    let markdown_fits = self.markdown.as_ref().is_some_and(|markdown| markdown.chars().count() <= MAX_MARKDOWN_CHARS);

    let attempts = [
      (self.media.is_some(), markdown_fits),
      (self.media.is_some(), false),
      (false, false),
    ];

    attempts
      .into_iter()
      .map(|(media, markdown)| self.payload(media, markdown))
      .find(|payload| payload.len() <= MAX_PAYLOAD_BYTES)
  }

  pub fn head_tags(&self) -> String {
    let mut tags = vec![
      format!(r#"<meta property="og:site_name" content="Rockbreaker">"#),
      format!(r#"<meta property="og:type" content="website">"#),
      format!(r#"<meta property="og:title" content="{}">"#, escape_html(&self.name)),
      format!(r#"<meta property="og:description" content="{}">"#, escape_html(&self.detail())),
      format!(r#"<meta property="og:url" content="{}">"#, escape_html(&self.page_url)),
    ];

    match &self.media {
      Some(Media::Image(url)) => {
        tags.push(format!(r#"<meta property="og:image" content="{}">"#, escape_html(url)));
        tags.push(r#"<meta name="twitter:card" content="summary_large_image">"#.to_string());
      }
      _ => tags.push(r#"<meta name="twitter:card" content="summary">"#.to_string()),
    }

    if let Some(payload) = self.component_embed() {
      tags.push(format!(r#"<script id="discord:component-embed" type="application/json">{payload}</script>"#));
    }

    tags.join("\n    ")
  }
}

fn parse_page_path(path: &str) -> Option<(String, String)> {
  let path = path.trim_matches('/');
  let (id, rest) = path.split_once('/').unwrap_or((path, ""));
  if !is_valid_id(id) {
    return None;
  }

  let subpath = percent_decode_str(rest).decode_utf8().ok()?;
  Some((id.to_string(), subpath.trim_matches('/').to_string()))
}

fn build_preview(state: &AppState, path: &str) -> Option<Preview> {
  let (id, subpath) = parse_page_path(path)?;
  let item = state.db.item(&id).ok()??;
  let rules = state.ignore_rules();
  let base = state.db.url_base();
  let base = base.trim_end_matches('/');

  let resolved = resolve_target(&item, &subpath, state.config.allow_external_symlinks, &rules)?;

  let page_url = if subpath.is_empty() {
    format!("{base}/{id}")
  } else {
    format!("{base}/{id}/{}", encode_path(&subpath))
  };

  let raw_url = |zip: bool| {
    let target = match (&item.source, subpath.is_empty()) {
      (ItemSource::File(path), true) => entry_name(path),
      _ => subpath.clone(),
    };

    match (target.is_empty(), zip) {
      (true, _) => format!("{base}/!/raw/{id}?zip=1"),
      (false, true) => format!("{base}/!/raw/{id}/{}?zip=1", encode_path(&target)),
      (false, false) => format!("{base}/!/raw/{id}/{}", encode_path(&target)),
    }
  };

  match resolved {
    Resolved::CollectionRoot => Some(Preview {
      name: "Shared items".to_string(),
      kind: Kind::Collection { names: item.source.paths().iter().map(|path| entry_name(path)).collect() },
      page_url,
      download_url: build_share_url(base, &item, true),
      media: None,
      markdown: None,
    }),
    Resolved::Path { path, .. } => {
      let name = Path::new(&subpath)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .or_else(|| item.root_name())
        .unwrap_or_else(|| id.clone());
      let metadata = std::fs::metadata(&path).ok()?;

      if metadata.is_dir() {
        return Some(Preview {
          name,
          kind: Kind::Folder,
          page_url,
          download_url: raw_url(true),
          media: None,
          markdown: None,
        });
      }

      let extension = extension(&name).unwrap_or_default();
      let download_url = raw_url(false);
      let media_url = format!("{download_url}?inline=1");

      let media = if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        Some(Media::Image(media_url))
      } else if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        Some(Media::Video(media_url))
      } else {
        None
      };

      let markdown = (MARKDOWN_EXTENSIONS.contains(&extension.as_str()) && metadata.len() <= MAX_MARKDOWN_FILE_BYTES)
        .then(|| std::fs::read_to_string(&path).ok())
        .flatten()
        .map(|text| text.replace("\r\n", "\n").trim().to_string())
        .filter(|text| !text.is_empty() && text.chars().count() <= MAX_MARKDOWN_CHARS);

      Some(Preview { name, kind: Kind::File { size: metadata.len() }, page_url, download_url, media, markdown })
    }
  }
}

pub async fn add_preview(state: &AppState, path: &str, html: String) -> String {
  let state = state.clone();
  let path = path.to_string();

  let tags = tokio::task::spawn_blocking(move || build_preview(&state, &path).map(|preview| preview.head_tags()))
    .await
    .ok()
    .flatten();

  match tags {
    Some(tags) => html.replacen("</head>", &format!("    {tags}\n  </head>"), 1),
    None => html,
  }
}
