use std::{
  fs,
  io::{self, Write},
  path::Path,
};

use serde::{Deserialize, Serialize};

use crate::paths::config_file_path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
  pub port: u16,
  pub allow_external_symlinks: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      port: 8080,
      allow_external_symlinks: false,
    }
  }
}

impl Config {
  pub fn load() -> (Config, Option<String>) {
    Self::load_from(&config_file_path())
  }

  fn load_from(path: &Path) -> (Config, Option<String>) {
    match fs::read_to_string(path) {
      Ok(content) => match toml::from_str(&content) {
        Ok(config) => (config, None),
        Err(e) => (Self::default(), Some(format!("Could not parse {}: {}", path.display(), e))),
      },
      Err(e) if e.kind() == io::ErrorKind::NotFound => {
        let config = Self::default();
        config.write_default(path);
        (config, None)
      }
      Err(e) => (Self::default(), Some(format!("Could not read {}: {}", path.display(), e))),
    }
  }

  fn write_default(&self, path: &Path) {
    let Ok(content) = toml::to_string_pretty(self) else {
      return;
    };

    if let Some(dir) = path.parent() {
      let _ = fs::create_dir_all(dir);
    }

    if let Ok(mut file) = fs::OpenOptions::new().write(true).create_new(true).open(path) {
      let _ = file.write_all(content.as_bytes());
    }
  }

  pub fn dashboard_url(&self, token: &str) -> String {
    format!("http://localhost:{}/?token={}", self.port, token)
  }
}
