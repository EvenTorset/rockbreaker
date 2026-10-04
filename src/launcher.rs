use std::{
  env, io,
  path::PathBuf,
  process::{Command as Process, Stdio},
  time::{Duration, Instant},
};

use arboard::Clipboard;

use crate::{
  cli::Command,
  client::{host_status, register_paths, HostStatus},
  config::Config,
  db,
  paths::db_file_path,
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn run(command: Command, config: Config) -> Result<(), String> {
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .map_err(|e| format!("Could not start the async runtime: {e}"))?;

  runtime.block_on(async {
    match command {
      Command::OpenDashboard => {
        let token = ensure_host_running(&config).await?;
        open_dashboard(&config, &token)
      }
      Command::Background => ensure_host_running(&config).await.map(drop),
      Command::Share { paths, direct, id, selection } => {
        let targets = paths
          .iter()
          .map(|path| dunce::canonicalize(path).map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e)))
          .collect::<Result<Vec<_>, _>>()?;
        let token = ensure_host_running(&config).await?;
        share(&config, &token, &targets, direct, id.as_deref(), selection).await
      }
      Command::Register | Command::Unregister | Command::Help | Command::Host => {
        unreachable!("handled before the launcher runs")
      }
    }
  })
}

fn read_token() -> Option<String> {
  db::read_token(&db_file_path())
}

async fn ensure_host_running(config: &Config) -> Result<String, String> {
  let token = read_token();
  match (host_status(config, token.as_deref()).await, token) {
    (HostStatus::Running, Some(token)) => return Ok(token),
    (HostStatus::NotRunning, _) => {}
    (HostStatus::Running, None) | (HostStatus::Rejected, _) => {
      return Err(format!(
        "Something is already listening on port {} but it did not accept the request.\n\n\
         If that is a Rockbreaker started by another user, or its token file was deleted, \
         quit it from its tray icon and try again.",
        config.port
      ));
    }
  }

  spawn_host().map_err(|e| format!("Could not start Rockbreaker in the background: {e}"))?;

  let deadline = Instant::now() + STARTUP_TIMEOUT;
  loop {
    tokio::time::sleep(STARTUP_POLL_INTERVAL).await;

    if let Some(token) = read_token() {
      if host_status(config, Some(&token)).await == HostStatus::Running {
        return Ok(token);
      }
    }

    if Instant::now() >= deadline {
      return Err(format!(
        "Rockbreaker did not start in the background. Port {} may already be in use by another program.",
        config.port
      ));
    }
  }
}

fn spawn_host() -> io::Result<()> {
  let mut process = Process::new(env::current_exe()?);
  process
    .arg("--host")
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());

  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    process.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
  }

  #[cfg(unix)]
  {
    use std::os::unix::process::CommandExt;
    // Its own process group, so Ctrl+C or closing the launching terminal doesn't take it down
    process.process_group(0);
  }

  process.spawn().map(drop)
}

fn open_dashboard(config: &Config, token: &str) -> Result<(), String> {
  let url = config.dashboard_url(token);
  open::that_detached(&url)
    .map_err(|e| format!("Could not open the dashboard in a browser: {e}\n\nDashboard: {url}"))
}

async fn share(
  config: &Config,
  token: &str,
  targets: &[PathBuf],
  direct: bool,
  id: Option<&str>,
  selection: bool,
) -> Result<(), String> {
  let response = register_paths(config, token, targets, direct, id, selection).await?;

  Clipboard::new()
    .and_then(|mut clipboard| clipboard.set_text(&response.share_url))
    .map_err(|e| format!("Could not copy the share link to the clipboard: {e}\n\nShare link: {}", response.share_url))
}
