use std::{io, path::Path};

pub fn reveal(path: &Path) -> io::Result<()> {
  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;

    std::process::Command::new("explorer.exe")
      .raw_arg(format!("/select,\"{}\"", path.display()))
      .spawn()
      .map(|_| ())
  }

  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open").arg("-R").arg(path).spawn().map(|_| ())
  }

  #[cfg(not(any(windows, target_os = "macos")))]
  {
    open::that_detached(path.parent().unwrap_or(path))
  }
}
