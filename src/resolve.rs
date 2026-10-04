use std::{
  fs,
  path::{Component, Path, PathBuf},
};

use crate::state::{HostedItem, ItemSource};

fn resolve_in_dir(root: &Path, subpath: &Path, allow_external: bool) -> Option<PathBuf> {
  let mut joined = root.to_path_buf();
  for component in subpath.components() {
    match component {
      Component::Normal(name) => joined.push(name),
      Component::CurDir => {}
      _ => return None,
    }
  }

  let real = dunce::canonicalize(joined).ok()?;
  (allow_external || real.starts_with(root)).then_some(real)
}

pub enum Resolved {
  CollectionRoot,
  Path { path: PathBuf, root: PathBuf },
}

pub fn resolve_target(item: &HostedItem, subpath: &str, allow_external: bool) -> Option<Resolved> {
  let subpath = Path::new(subpath);

  match &item.source {
    ItemSource::File(path) => {
      (subpath.as_os_str().is_empty() && path.exists()).then(|| Resolved::Path { path: path.clone(), root: path.clone() })
    }
    ItemSource::Directory(root) => {
      let path = resolve_in_dir(root, subpath, allow_external)?;
      Some(Resolved::Path { path, root: root.clone() })
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
        let path = resolve_in_dir(entry, rest, allow_external)?;
        Some(Resolved::Path { path, root: entry.clone() })
      } else {
        (rest.as_os_str().is_empty() && entry.exists()).then(|| Resolved::Path { path: entry.clone(), root: entry.clone() })
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
