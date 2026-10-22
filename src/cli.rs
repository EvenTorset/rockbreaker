use std::path::PathBuf;

use crate::state::is_valid_id;

pub const USAGE: &str = "\
Rockbreaker

Usage:
  rockbreaker                       Start Rockbreaker in the background if it isn't running, then open the dashboard
  rockbreaker --background          Start Rockbreaker in the background without opening the dashboard
  rockbreaker [options] <path>...   Share files and folders (several are shared together as one item)
  rockbreaker --register            Add Rockbreaker to the file explorer context menu
  rockbreaker --unregister          Remove Rockbreaker from the file explorer context menu

Sharing options:
  --id <id>   Host the item at this ID instead of a random one (letters, numbers, _ and -)
  --direct    Copy the direct download link instead of the link to the landing page
  --no-copy   Don't copy anything, the item can be found in the dashboard
  --open      Open the link to the landing page in a browser instead of copying it
";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AfterSharing {
  #[default]
  CopyShareLink,
  CopyDirectLink,
  Nothing,
  OpenShareLink,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
  /// Make sure the host is running, then open the dashboard.
  OpenDashboard,
  /// Make sure the host is running, and nothing else.
  Background,
  /// Make sure the host is running, then share paths through it.
  Share {
    paths: Vec<PathBuf>,
    after: AfterSharing,
    id: Option<String>,
    selection: bool,
  },
  Register,
  Unregister,
  Help,
  /// Internal: run as the host process itself. This is what `Background` spawns.
  Host,
}

pub fn parse(args: &[String]) -> Result<Command, String> {
  if let [only] = args {
    match only.as_str() {
      "--register" => return Ok(Command::Register),
      "--unregister" => return Ok(Command::Unregister),
      "--host" => return Ok(Command::Host),
      "--help" | "-h" => return Ok(Command::Help),
      _ => {}
    }
  }

  let mut direct = false;
  let mut no_copy = false;
  let mut open = false;
  let mut background = false;
  let mut selection = false;
  let mut id: Option<String> = None;
  let mut paths: Vec<PathBuf> = Vec::new();

  let mut args = args.iter();
  while let Some(arg) = args.next() {
    match arg.as_str() {
      "--direct" => direct = true,
      "--no-copy" => no_copy = true,
      "--open" => open = true,
      "--background" => background = true,
      "--selection" => selection = true,
      "--register" | "--unregister" | "--host" | "--help" | "-h" => {
        return Err(format!("{arg} can't be combined with other arguments."));
      }
      "--id" => {
        let value = args.next().ok_or("--id needs an ID after it.")?;
        id = Some(check_id(value)?);
      }
      option if option.starts_with("--id=") => id = Some(check_id(&option["--id=".len()..])?),
      option if option.starts_with("--") => return Err(format!("Unknown option: {option}")),
      _ => paths.push(PathBuf::from(arg)),
    }
  }

  if [direct, no_copy, open].into_iter().filter(|chosen| *chosen).count() > 1 {
    return Err("--direct, --no-copy and --open can't be combined.".to_string());
  }

  if paths.is_empty() {
    return match (direct, no_copy, open, id.is_some(), selection, background) {
      (true, ..) => Err("--direct needs a path to share.".to_string()),
      (_, true, ..) => Err("--no-copy needs a path to share.".to_string()),
      (_, _, true, ..) => Err("--open needs a path to share.".to_string()),
      (_, _, _, true, ..) => Err("--id needs a path to share.".to_string()),
      (_, _, _, _, true, _) => Err("--selection needs a path to share.".to_string()),
      (_, _, _, _, _, true) => Ok(Command::Background),
      _ => Ok(Command::OpenDashboard),
    };
  }

  if selection && id.is_some() {
    return Err("--id can't be used with --selection.".to_string());
  }

  let after = match (direct, no_copy, open) {
    (true, ..) => AfterSharing::CopyDirectLink,
    (_, true, _) => AfterSharing::Nothing,
    (_, _, true) => AfterSharing::OpenShareLink,
    _ => AfterSharing::CopyShareLink,
  };

  Ok(Command::Share { paths, after, id, selection })
}

fn check_id(id: &str) -> Result<String, String> {
  if is_valid_id(id) {
    Ok(id.to_string())
  } else {
    Err(format!("'{id}' isn't a valid ID. Use only letters, numbers, underscores and dashes."))
  }
}
