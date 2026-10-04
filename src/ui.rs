use std::io::{self, IsTerminal};

use rfd::{MessageButtons, MessageDialog, MessageLevel};

#[derive(Clone, Copy)]
pub enum Level {
  Info,
  Warning,
  Error,
}

#[cfg(windows)]
fn attach_parent_console() {
  use std::{fs::OpenOptions, os::windows::io::IntoRawHandle, sync::Once};

  use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    System::Console::{
      AttachConsole, GetStdHandle, SetStdHandle, ATTACH_PARENT_PROCESS, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    },
  };

  static ATTACH: Once = Once::new();

  ATTACH.call_once(|| unsafe {
    if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
      return;
    }

    for std_handle in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
      let current = GetStdHandle(std_handle);

      if !current.is_null() && current != INVALID_HANDLE_VALUE {
        continue;
      }

      if let Ok(console) = OpenOptions::new().write(true).open("CONOUT$") {
        SetStdHandle(std_handle, console.into_raw_handle());
      }
    }
  });
}

pub fn show(level: Level, message: &str) {
  #[cfg(windows)]
  attach_parent_console();

  match level {
    Level::Info if io::stdout().is_terminal() => return println!("{message}"),
    Level::Warning | Level::Error if io::stderr().is_terminal() => return eprintln!("{message}"),
    _ => {}
  }

  MessageDialog::new()
    .set_title("Rockbreaker")
    .set_description(message)
    .set_level(match level {
      Level::Info => MessageLevel::Info,
      Level::Warning => MessageLevel::Warning,
      Level::Error => MessageLevel::Error,
    })
    .set_buttons(MessageButtons::Ok)
    .show();
}

pub fn info(message: &str) {
  show(Level::Info, message);
}

pub fn warning(message: &str) {
  show(Level::Warning, message);
}

pub fn error(message: &str) {
  show(Level::Error, message);
}
