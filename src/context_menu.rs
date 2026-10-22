use std::io;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

// MARK: Options

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOption {
  CopyLink,
  CopyDirect,
  Share,
  ShareOpen,
}

impl MenuOption {
  pub const ALL: [MenuOption; 4] = [Self::CopyLink, Self::CopyDirect, Self::Share, Self::ShareOpen];

  pub fn id(self) -> &'static str {
    match self {
      Self::CopyLink => "copy_link",
      Self::CopyDirect => "copy_direct",
      Self::Share => "share",
      Self::ShareOpen => "share_open",
    }
  }

  pub fn from_id(id: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|option| option.id() == id)
  }

  pub fn submenu_label(self) -> &'static str {
    match self {
      Self::CopyLink => "Copy share link",
      Self::CopyDirect => "Copy direct download link",
      Self::Share => "Share",
      Self::ShareOpen => "Share and open",
    }
  }

  pub fn single_label(self) -> &'static str {
    match self {
      Self::CopyLink => "Rockbreaker: Copy share link",
      Self::CopyDirect => "Rockbreaker: Copy direct download link",
      Self::Share => "Share with Rockbreaker",
      Self::ShareOpen => "Rockbreaker: Share and open",
    }
  }

  pub fn description(self) -> &'static str {
    match self {
      Self::CopyLink => "Shares it and copies the link to its page.",
      Self::CopyDirect => "Shares it and copies a link that downloads it right away.",
      Self::Share => "Shares it without copying anything. It can be found in the dashboard.",
      Self::ShareOpen => "Shares it and opens the link to its page.",
    }
  }

  fn arguments(self) -> &'static str {
    match self {
      Self::CopyLink => "",
      Self::CopyDirect => "--direct",
      Self::Share => "--no-copy",
      Self::ShareOpen => "--open",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuSetting {
  pub option: MenuOption,
  pub enabled: bool,
}

pub fn default_settings() -> Vec<MenuSetting> {
  MenuOption::ALL.into_iter().map(|option| MenuSetting { option, enabled: true }).collect()
}

pub fn complete_settings(settings: impl IntoIterator<Item = MenuSetting>) -> Vec<MenuSetting> {
  let mut complete: Vec<MenuSetting> = Vec::new();
  for setting in settings {
    if complete.iter().all(|existing| existing.option != setting.option) {
      complete.push(setting);
    }
  }

  for option in MenuOption::ALL {
    if complete.iter().all(|existing| existing.option != option) {
      complete.push(MenuSetting { option, enabled: true });
    }
  }
  complete
}

pub fn enabled_options(settings: &[MenuSetting]) -> Vec<MenuOption> {
  settings.iter().filter(|setting| setting.enabled).map(|setting| setting.option).collect()
}

pub fn settings_to_string(settings: &[MenuSetting]) -> String {
  settings
    .iter()
    .map(|setting| format!("{}:{}", setting.option.id(), if setting.enabled { 1 } else { 0 }))
    .collect::<Vec<_>>()
    .join(",")
}

pub fn settings_from_string(text: &str) -> Vec<MenuSetting> {
  let parsed = text.split(',').filter_map(|part| {
    let (id, enabled) = part.trim().split_once(':')?;
    Some(MenuSetting { option: MenuOption::from_id(id)?, enabled: enabled != "0" })
  });

  let settings = complete_settings(parsed);
  if enabled_options(&settings).is_empty() { default_settings() } else { settings }
}

// MARK: Layout

#[cfg_attr(not(windows), allow(dead_code))]
pub const SUBMENU_LABEL: &str = "Share with Rockbreaker...";

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum Layout {
  Single(MenuOption),
  Submenu(Vec<MenuOption>),
}

#[cfg_attr(not(windows), allow(dead_code))]
pub fn layout(options: &[MenuOption]) -> Option<Layout> {
  match options {
    [] => None,
    [only] => Some(Layout::Single(*only)),
    several => Some(Layout::Submenu(several.to_vec())),
  }
}

#[cfg_attr(not(windows), allow(dead_code))]
fn command_line(app_path: &str, option: MenuOption, placeholder: &str) -> String {
  match option.arguments() {
    "" => format!("\"{app_path}\" --selection \"{placeholder}\""),
    arguments => format!("\"{app_path}\" --selection {arguments} \"{placeholder}\""),
  }
}

#[cfg_attr(not(windows), allow(dead_code))]
fn verb_name(index: usize, option: MenuOption) -> String {
  format!("{}_{}", index + 1, option.id())
}

// MARK: Registry

#[cfg(windows)]
const MENU_KEYS: [(&str, &str); 3] = [
  (r"*\shell\Rockbreaker", "%1"),
  (r"Directory\shell\Rockbreaker", "%1"),
  (r"Directory\Background\shell\Rockbreaker", "%V"),
];

#[cfg(windows)]
fn app_path() -> io::Result<String> {
  Ok(std::env::current_exe()?.to_string_lossy().to_string())
}

#[cfg(windows)]
pub fn register(settings: &[MenuSetting]) -> io::Result<()> {
  let Some(layout) = layout(&enabled_options(settings)) else {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "At least one option has to be turned on."));
  };

  let app_path = app_path()?;
  let (classes, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(r"Software\Classes")?;

  for (key, placeholder) in MENU_KEYS {
    delete_subkey_all_if_present(&classes, key)?;
    write_menu(&classes, key, &layout, &app_path, placeholder)?;
  }

  Ok(())
}

#[cfg(windows)]
fn write_menu(classes: &RegKey, key: &str, layout: &Layout, app_path: &str, placeholder: &str) -> io::Result<()> {
  let (menu_key, _) = classes.create_subkey(key)?;
  menu_key.set_value("Icon", &app_path)?;
  menu_key.set_value("MultiSelectModel", &"Player")?;

  match layout {
    Layout::Single(option) => {
      menu_key.set_value("MUIVerb", &option.single_label())?;

      let (command_key, _) = menu_key.create_subkey("command")?;
      command_key.set_value("", &command_line(app_path, *option, placeholder))?;
    }
    Layout::Submenu(options) => {
      menu_key.set_value("MUIVerb", &SUBMENU_LABEL)?;
      menu_key.set_value("SubCommands", &"")?;

      let (shell_key, _) = menu_key.create_subkey("shell")?;
      for (index, option) in options.iter().enumerate() {
        let (verb_key, _) = shell_key.create_subkey(verb_name(index, *option))?;
        verb_key.set_value("", &option.submenu_label())?;
        verb_key.set_value("MultiSelectModel", &"Player")?;

        let (command_key, _) = verb_key.create_subkey("command")?;
        command_key.set_value("", &command_line(app_path, *option, placeholder))?;
      }
    }
  }

  Ok(())
}

#[cfg(windows)]
pub fn unregister() -> io::Result<()> {
  let classes = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Classes", KEY_ALL_ACCESS);
  let Ok(classes) = classes else {
    return Ok(());
  };

  for (key, _) in MENU_KEYS {
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
pub fn is_registered(settings: &[MenuSetting]) -> bool {
  let (Ok(app_path), Some(layout)) = (app_path(), layout(&enabled_options(settings))) else {
    return false;
  };

  let classes = RegKey::predef(HKEY_CURRENT_USER);
  MENU_KEYS
    .iter()
    .all(|(key, placeholder)| menu_matches(&classes, key, &layout, &app_path, placeholder))
}

#[cfg(windows)]
fn menu_matches(classes: &RegKey, key: &str, layout: &Layout, app_path: &str, placeholder: &str) -> bool {
  let Ok(menu_key) = classes.open_subkey(format!(r"Software\Classes\{key}")) else {
    return false;
  };

  let text = |key: &RegKey, name: &str| key.get_value::<String, _>(name).ok();
  let command_of = |key: &RegKey| key.open_subkey("command").ok().and_then(|command| text(&command, ""));

  match layout {
    Layout::Single(option) => {
      text(&menu_key, "MUIVerb").as_deref() == Some(option.single_label())
        && text(&menu_key, "SubCommands").is_none()
        && command_of(&menu_key) == Some(command_line(app_path, *option, placeholder))
    }
    Layout::Submenu(options) => {
      let Ok(shell_key) = menu_key.open_subkey("shell") else {
        return false;
      };

      text(&menu_key, "MUIVerb").as_deref() == Some(SUBMENU_LABEL)
        && shell_key.enum_keys().filter_map(Result::ok).count() == options.len()
        && options.iter().enumerate().all(|(index, option)| {
          shell_key.open_subkey(verb_name(index, *option)).is_ok_and(|verb_key| {
            text(&verb_key, "").as_deref() == Some(option.submenu_label())
              && command_of(&verb_key) == Some(command_line(app_path, *option, placeholder))
          })
        })
    }
  }
}

#[cfg(not(windows))]
pub fn register(_settings: &[MenuSetting]) -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Registration is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn unregister() -> io::Result<()> {
  Err(io::Error::new(io::ErrorKind::Unsupported, "Unregistration is only supported on Windows."))
}

#[cfg(not(windows))]
pub fn is_registered(_settings: &[MenuSetting]) -> bool {
  false
}
