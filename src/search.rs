use std::{
  fs,
  path::{Path, PathBuf},
  time::{Duration, Instant},
};

use crate::{
  api::modified_millis,
  ignore_rules::{IgnoreRules, Scope},
  resolve::{resolve_entry, resolve_target, Resolved},
  state::{entry_name, HostedItem},
};

pub const MAX_RESULTS: usize = 1000;
const MAX_SCANNED: usize = 500_000;
const MAX_DURATION: Duration = Duration::from_secs(5);

pub struct Found {
  pub name: String,
  pub is_dir: bool,
  pub size: Option<u64>,
  pub modified: Option<i64>,
  pub subpath: String,
}

pub struct Outcome {
  pub found: Vec<Found>,
  pub truncated: bool,
}

struct Limits {
  max_results: usize,
  max_scanned: usize,
  max_duration: Duration,
}

struct Search<'a> {
  needle: String,
  allow_external: bool,
  rules: &'a IgnoreRules,
  limits: Limits,
  started: Instant,
  scanned: usize,
  outcome: Outcome,
}

impl Search<'_> {
  fn should_stop(&mut self) -> bool {
    let reached = self.outcome.found.len() >= self.limits.max_results
      || self.scanned >= self.limits.max_scanned
      || self.started.elapsed() >= self.limits.max_duration;

    self.outcome.truncated |= reached;
    reached
  }

  fn consider(&mut self, name: &str, is_dir: bool, metadata: &fs::Metadata, subpath: &str) {
    if !name.to_lowercase().contains(&self.needle) {
      return;
    }

    self.outcome.found.push(Found {
      name: name.to_string(),
      is_dir,
      size: metadata.is_file().then_some(metadata.len()),
      modified: modified_millis(metadata),
      subpath: subpath.to_string(),
    });
  }

  fn walk(&mut self, root: &Path, dir: &Path, prefix: &str, scope: &Scope, ancestors: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
      return;
    };

    for entry in entries.flatten() {
      if self.should_stop() {
        return;
      }

      let Some(resolved) = resolve_entry(root, &entry, self.allow_external) else {
        continue;
      };
      let file_name = entry.file_name();
      let is_dir = resolved.metadata.is_dir();
      if scope.is_ignored(&file_name, is_dir) {
        continue;
      }

      self.scanned += 1;
      let name = file_name.to_string_lossy().to_string();
      let subpath = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
      self.consider(&name, is_dir, &resolved.metadata, &subpath);

      if is_dir && !ancestors.contains(&resolved.path) {
        let child_scope = self.rules.enter(scope, &entry.path(), &file_name);
        ancestors.push(resolved.path.clone());
        self.walk(root, &resolved.path, &subpath, &child_scope, ancestors);
        ancestors.pop();
      }
    }
  }
}

pub fn search(
  item: &HostedItem,
  subpath: &str,
  query: &str,
  allow_external: bool,
  rules: &IgnoreRules,
) -> Option<Outcome> {
  search_with_limits(
    item,
    subpath,
    query,
    allow_external,
    rules,
    Limits { max_results: MAX_RESULTS, max_scanned: MAX_SCANNED, max_duration: MAX_DURATION },
  )
}

fn search_with_limits(
  item: &HostedItem,
  subpath: &str,
  query: &str,
  allow_external: bool,
  rules: &IgnoreRules,
  limits: Limits,
) -> Option<Outcome> {
  let resolved = resolve_target(item, subpath, allow_external, rules)?;

  let mut search = Search {
    needle: query.trim().to_lowercase(),
    allow_external,
    rules,
    limits,
    started: Instant::now(),
    scanned: 0,
    outcome: Outcome { found: Vec::new(), truncated: false },
  };

  match resolved {
    Resolved::CollectionRoot => {
      for path in item.source.paths() {
        if search.should_stop() {
          break;
        }
        let Ok(metadata) = fs::metadata(path) else {
          continue;
        };

        let name = entry_name(path);
        search.consider(&name, metadata.is_dir(), &metadata, &name);

        if metadata.is_dir() {
          let scope = rules.root_scope(path);
          search.walk(path, path, &name, &scope, &mut vec![path.to_path_buf()]);
        }
      }
    }
    Resolved::Path { path, root, relative } if path.is_dir() => {
      let scope = rules.scope_for(&root, &relative);
      let prefix = subpath.trim_matches('/');
      search.walk(&root, &path, prefix, &scope, &mut vec![path.clone()]);
    }
    Resolved::Path { .. } => return None,
  }

  Some(search.outcome)
}
