use std::io;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
const MENU_KEYS: [&str; 2] = [r"*\shell\Rockbreaker", r"Directory\shell\Rockbreaker"];

// Explorer lists the entries of a submenu in the order of their key names
#[cfg(windows)]
const LINK_VERB: &str = "1_link";
#[cfg(windows)]
const DIRECT_VERB: &str = "2_direct";

#[cfg(windows)]
fn app_path() -> io::Result<String> {
  Ok(std::env::current_exe()?.to_string_lossy().to_string())
}

#[cfg(windows)]
fn link_command(app_path: &str) -> String {
  format!("\"{app_path}\" --selection \"%1\"")
}

#[cfg(windows)]
fn direct_command(app_path: &str) -> String {
  format!("\"{app_path}\" --selection --direct \"%1\"")
}

#[cfg(windows)]
pub fn register() -> io::Result<()> {
  let app_path = app_path()?;
  let (classes, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(r"Software\Classes")?;

  for key in MENU_KEYS {
    delete_subkey_all_if_present(&classes, key)?;
    register_shell_submenu(&classes, key, &app_path)?;
  }

  Ok(())
}

#[cfg(windows)]
fn register_shell_submenu(classes: &RegKey, base_path: &str, app_path: &str) -> io::Result<()> {
  let (parent_key, _) = classes.create_subkey(base_path)?;
  parent_key.set_value("MUIVerb", &"Share with Rockbreaker...")?;
  parent_key.set_value("Icon", &app_path)?;
  parent_key.set_value("SubCommands", &"")?;
  parent_key.set_value("MultiSelectModel", &"Player")?;

  let (shell_key, _) = parent_key.create_subkey("shell")?;

  let verbs = [
    (LINK_VERB, "Copy share link", link_command(app_path)),
    (DIRECT_VERB, "Copy direct download link", direct_command(app_path)),
  ];
  for (verb, label, command) in verbs {
    let (verb_key, _) = shell_key.create_subkey(verb)?;
    verb_key.set_value("", &label)?;
    verb_key.set_value("MultiSelectModel", &"Player")?;
    let (command_key, _) = verb_key.create_subkey("command")?;
    command_key.set_value("", &command)?;
  }

  Ok(())
}

#[cfg(windows)]
pub fn unregister() -> io::Result<()> {
  let classes = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Classes", KEY_ALL_ACCESS);
  let Ok(classes) = classes else {
    return Ok(());
  };

  for key in MENU_KEYS {
    delete_subkey_all_if_present(&classes, key)?;
  }

  Ok(())
}

#[cfg(windows)]
fn delete_subkey_all_if_present(root: &RegKey, path: &str) -> io::Result<()> {
  match root.delete_subkey_all(path) {
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
    other => other,
  }
}

#[cfg(windows)]
pub fn is_registered() -> bool {
  let Ok(app_path) = app_path() else {
    return false;
  };

  let classes = RegKey::predef(HKEY_CURRENT_USER);
  let command_is = |key: &str, verb: &str, expected: &str| {
    classes
      .open_subkey(format!(r"Software\Classes\{key}\shell\{verb}\command"))
      .and_then(|command| command.get_value::<String, _>(""))
      .is_ok_and(|value| value == expected)
  };

  MENU_KEYS.iter().all(|key| {
    command_is(key, LINK_VERB, &link_command(&app_path)) && command_is(key, DIRECT_VERB, &direct_command(&app_path))
  })
}

#[cfg(not(windows))]
pub fn register() -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Registration is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn unregister() -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Unregistration is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn is_registered() -> bool {
  false
}
