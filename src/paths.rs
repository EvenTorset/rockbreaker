use std::{env, path::PathBuf};

const DB_FILE_NAME: &str = "rockbreaker.db";
const CONFIG_FILE_NAME: &str = "rockbreaker.config.toml";

#[cfg(windows)]
fn exe_dir() -> PathBuf {
  env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
    .unwrap_or_default()
}

pub fn db_file_path() -> PathBuf {
  #[cfg(windows)]
  {
    exe_dir().join(DB_FILE_NAME)
  }
  #[cfg(not(windows))]
  {
    dirs::data_local_dir()
      .unwrap_or_else(|| env::temp_dir())
      .join("rockbreaker")
      .join(DB_FILE_NAME)
  }
}

pub fn config_file_path() -> PathBuf {
  #[cfg(windows)]
  {
    exe_dir().join(CONFIG_FILE_NAME)
  }
  #[cfg(not(windows))]
  {
    dirs::config_dir()
      .unwrap_or_else(|| env::temp_dir())
      .join("rockbreaker")
      .join(CONFIG_FILE_NAME)
  }
}
