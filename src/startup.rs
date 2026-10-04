use std::io;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const VALUE_NAME: &str = "Rockbreaker";

#[cfg(windows)]
fn startup_command() -> io::Result<String> {
  let app_path = std::env::current_exe()?;
  Ok(format!("\"{}\" --background", app_path.to_string_lossy()))
}

#[cfg(windows)]
pub fn enable() -> io::Result<()> {
  let command = startup_command()?;
  let (run_key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(RUN_KEY)?;
  run_key.set_value(VALUE_NAME, &command)
}

#[cfg(windows)]
pub fn disable() -> io::Result<()> {
  let Ok(run_key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) else {
    return Ok(());
  };

  match run_key.delete_value(VALUE_NAME) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
    other => other,
  }
}

#[cfg(windows)]
pub fn is_enabled() -> bool {
  let Ok(expected) = startup_command() else {
    return false;
  };

  RegKey::predef(HKEY_CURRENT_USER)
    .open_subkey(RUN_KEY)
    .and_then(|run_key| run_key.get_value::<String, _>(VALUE_NAME))
    .is_ok_and(|value| value == expected)
}

#[cfg(not(windows))]
pub fn enable() -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Running on startup is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn disable() -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Running on startup is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn is_enabled() -> bool {
  false
}
