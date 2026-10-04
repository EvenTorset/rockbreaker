use std::{path::PathBuf, time::Duration};

use crate::{
  api::{RegisterRequest, RegisterResponse},
  config::Config,
  token::TOKEN_HEADER,
};

#[derive(Debug, PartialEq, Eq)]
pub enum HostStatus {
  Running,
  NotRunning,
  Rejected,
}

pub async fn host_status(config: &Config, token: Option<&str>) -> HostStatus {
  let url = format!("http://127.0.0.1:{}/!/api/dashboard", config.port);

  let mut request = reqwest::Client::new().get(&url).timeout(Duration::from_secs(2));
  if let Some(token) = token {
    request = request.header(TOKEN_HEADER, token);
  }

  match request.send().await {
    Ok(response) if response.status().is_success() => HostStatus::Running,
    Ok(_) => HostStatus::Rejected,
    Err(_) => HostStatus::NotRunning,
  }
}

const SELECTION_TIMEOUT: Duration = Duration::from_secs(20);

pub async fn register_paths(
  config: &Config,
  token: &str,
  paths: &[PathBuf],
  direct: bool,
  id: Option<&str>,
  selection: bool,
) -> Result<RegisterResponse, String> {
  let url = format!("http://127.0.0.1:{}/!/api/register", config.port);
  let payload = RegisterRequest {
    paths: paths.iter().map(|path| path.to_string_lossy().to_string()).collect(),
    direct,
    id: id.map(str::to_string),
    selection,
  };

  let response = reqwest::Client::new()
    .post(&url)
    .timeout(if selection { SELECTION_TIMEOUT } else { Duration::from_secs(5) })
    .header(TOKEN_HEADER, token)
    .json(&payload)
    .send()
    .await
    .map_err(|e| format!("Could not reach the Rockbreaker host: {e}"))?;

  let status = response.status();
  if !status.is_success() {
    let reason = response.text().await.unwrap_or_default();
    return Err(if reason.is_empty() {
      format!("The Rockbreaker host rejected the request (HTTP {}).", status.as_u16())
    } else {
      reason
    });
  }

  response
    .json::<RegisterResponse>()
    .await
    .map_err(|e| format!("The Rockbreaker host sent an unexpected response: {e}"))
}
