use std::{
  fs,
  path::{Component, Path, PathBuf},
};

use crate::{
  ignore_rules::IgnoreRules,
  state::{HostedItem, ItemSource},
};

fn resolve_in_dir(
  root: &Path,
  subpath: &Path,
  allow_external: bool,
  rules: &IgnoreRules,
) -> Option<(PathBuf, PathBuf)> {
  let mut joined = root.to_path_buf();
  let mut relative = PathBuf::new();
  for component in subpath.components() {
    match component {
      Component::Normal(name) => {
        joined.push(name);
        relative.push(name);
      }
      Component::CurDir => {}
      _ => return None,
    }
  }

  let real = dunce::canonicalize(joined).ok()?;
  if !allow_external && !real.starts_with(root) {
    return None;
  }
  if !relative.as_os_str().is_empty() && rules.is_ignored(root, &relative, real.is_dir()) {
    return None;
  }

  Some((real, relative))
}

pub enum Resolved {
  CollectionRoot,
  Path {
    path: PathBuf,
    root: PathBuf,
    relative: PathBuf,
  },
}

pub fn resolve_target(
  item: &HostedItem,
  subpath: &str,
  allow_external: bool,
  rules: &IgnoreRules,
) -> Option<Resolved> {
  let subpath = Path::new(subpath);

  match &item.source {
    ItemSource::File(path) => {
      (subpath.as_os_str().is_empty() && path.exists()).then(|| Resolved::Path {
        path: path.clone(),
        root: path.clone(),
        relative: PathBuf::new(),
      })
    }
    ItemSource::Directory(root) => {
      let (path, relative) = resolve_in_dir(root, subpath, allow_external, rules)?;
      Some(Resolved::Path { path, root: root.clone(), relative })
    }
    ItemSource::Collection(paths) => {
      let mut components = subpath.components();
      let Some(first) = components.next() else {
        return Some(Resolved::CollectionRoot);
      };
      let Component::Normal(name) = first else {
        return None;
      };

      let entry = paths.iter().find(|path| path.file_name() == Some(name))?;
      let rest = components.as_path();

      if entry.is_dir() {
        let (path, relative) = resolve_in_dir(entry, rest, allow_external, rules)?;
        Some(Resolved::Path { path, root: entry.clone(), relative })
      } else {
        (rest.as_os_str().is_empty() && entry.exists()).then(|| Resolved::Path {
          path: entry.clone(),
          root: entry.clone(),
          relative: PathBuf::new(),
        })
      }
    }
  }
}

pub struct ResolvedEntry {
  pub path: PathBuf,
  pub metadata: fs::Metadata,
}

pub fn resolve_entry(root: &Path, entry: &fs::DirEntry, allow_external: bool) -> Option<ResolvedEntry> {
  let (path, metadata) = if entry.file_type().ok()?.is_symlink() {
    let real = dunce::canonicalize(entry.path()).ok()?;
    if !allow_external && !real.starts_with(root) {
      return None;
    }
    let metadata = fs::metadata(&real).ok()?;
    (real, metadata)
  } else {
    (entry.path(), entry.metadata().ok()?)
  };

  (metadata.is_dir() || metadata.is_file()).then_some(ResolvedEntry { path, metadata })
}
