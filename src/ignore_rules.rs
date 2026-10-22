use std::{
  collections::HashMap,
  ffi::{OsStr, OsString},
  fs,
  path::{Component, Path, PathBuf},
  sync::{Arc, Mutex},
  time::SystemTime,
};

use ignore::{
  gitignore::{Gitignore, GitignoreBuilder},
  Match,
};

pub fn validate_pattern(pattern: &str) -> Result<(), String> {
  GitignoreBuilder::new(".")
    .add_line(None, pattern)
    .map(|_| ())
    .map_err(|error| {
      let message = error.to_string();
      match message.split_once("': ") {
        Some((_, reason)) => reason.to_string(),
        None => message,
      }
    })
}

pub fn normalize_patterns<S: AsRef<str>>(patterns: &[S]) -> Vec<String> {
  patterns
    .iter()
    .map(|pattern| pattern.as_ref().trim().to_string())
    .filter(|pattern| !pattern.is_empty())
    .collect()
}

struct CachedGitignore {
  stamp: Option<(Option<SystemTime>, u64)>,
  rules: Option<Arc<Gitignore>>,
}

pub struct IgnoreRules {
  global: Option<Arc<Gitignore>>,
  respect_gitignore: bool,
  gitignores: Mutex<HashMap<PathBuf, CachedGitignore>>,
}

impl IgnoreRules {
  pub fn new(patterns: &[String], respect_gitignore: bool) -> Self {
    let mut builder = GitignoreBuilder::new(".");
    let mut any = false;
    for pattern in patterns {
      any |= builder.add_line(None, pattern).is_ok();
    }

    Self {
      global: any.then(|| builder.build().ok().map(Arc::new)).flatten(),
      respect_gitignore,
      gitignores: Mutex::new(HashMap::new()),
    }
  }

  fn gitignore_in(&self, dir: &Path) -> Option<Arc<Gitignore>> {
    let file = dir.join(".gitignore");
    let stamp = fs::metadata(&file)
      .ok()
      .filter(fs::Metadata::is_file)
      .map(|metadata| (metadata.modified().ok(), metadata.len()));

    let mut cache = self.gitignores.lock().unwrap();
    if let Some(cached) = cache.get(dir) {
      if cached.stamp == stamp {
        return cached.rules.clone();
      }
    }

    let rules = stamp.and_then(|_| {
      let mut builder = GitignoreBuilder::new(".");
      let _ = builder.add(&file);
      builder.build().ok().map(Arc::new)
    });

    cache.insert(dir.to_path_buf(), CachedGitignore { stamp, rules: rules.clone() });
    rules
  }

  pub fn root_scope(&self, root: &Path) -> Scope {
    let chain = match self.respect_gitignore.then(|| self.gitignore_in(root)).flatten() {
      Some(rules) => vec![(0, rules)],
      None => Vec::new(),
    };

    Scope { rel_dir: Vec::new(), global: self.global.clone(), chain }
  }

  pub fn enter(&self, parent: &Scope, dir: &Path, name: &OsStr) -> Scope {
    let mut scope = parent.clone();
    scope.rel_dir.push(name.to_os_string());

    if self.respect_gitignore {
      if let Some(rules) = self.gitignore_in(dir) {
        scope.chain.push((scope.rel_dir.len(), rules));
      }
    }
    scope
  }

  pub fn scope_for(&self, root: &Path, relative: &Path) -> Scope {
    let mut scope = self.root_scope(root);
    let mut dir = root.to_path_buf();

    for component in relative.components() {
      if let Component::Normal(name) = component {
        dir.push(name);
        scope = self.enter(&scope, &dir, name);
      }
    }
    scope
  }

  pub fn is_ignored(&self, root: &Path, relative: &Path, is_dir: bool) -> bool {
    let names: Vec<&OsStr> = relative
      .components()
      .filter_map(|component| match component {
        Component::Normal(name) => Some(name),
        _ => None,
      })
      .collect();

    let mut scope = self.root_scope(root);
    let mut dir = root.to_path_buf();

    for (index, name) in names.iter().enumerate() {
      let is_last = index + 1 == names.len();
      dir.push(name);

      if scope.is_ignored(name, !is_last || is_dir) {
        return true;
      }
      if !is_last {
        scope = self.enter(&scope, &dir, name);
      }
    }

    false
  }
}

#[derive(Clone)]
pub struct Scope {
  rel_dir: Vec<OsString>,
  global: Option<Arc<Gitignore>>,
  chain: Vec<(usize, Arc<Gitignore>)>,
}

impl Scope {
  pub fn is_ignored(&self, name: &OsStr, is_dir: bool) -> bool {
    if let Some(global) = &self.global {
      let path: PathBuf = self.rel_dir.iter().map(OsString::as_os_str).chain([name]).collect();
      if global.matched(path, is_dir).is_ignore() {
        return true;
      }
    }

    let mut ignored = false;
    for (depth, rules) in &self.chain {
      let path: PathBuf = self.rel_dir[*depth..].iter().map(OsString::as_os_str).chain([name]).collect();
      match rules.matched(path, is_dir) {
        Match::Ignore(_) => ignored = true,
        Match::Whitelist(_) => ignored = false,
        Match::None => {}
      }
    }
    ignored
  }
}
