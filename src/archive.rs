use std::{
  fs::{self, File},
  io::{self, Seek, Write},
  path::{Path, PathBuf},
};

use axum::body::{Body, Bytes};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::resolve::resolve_entry;

const ZIP_CHUNK_SIZE: usize = 64 * 1024;

type ZipChunk = io::Result<Bytes>;

struct ChannelWriter {
  tx: mpsc::Sender<ZipChunk>,
  buffer: Vec<u8>,
}

impl ChannelWriter {
  fn send_buffered(&mut self) -> io::Result<()> {
    if self.buffer.is_empty() {
      return Ok(());
    }

    let chunk = Bytes::from(std::mem::replace(&mut self.buffer, Vec::with_capacity(ZIP_CHUNK_SIZE)));
    self
      .tx
      .blocking_send(Ok(chunk))
      .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))
  }
}

impl Write for ChannelWriter {
  fn write(&mut self, data: &[u8]) -> io::Result<usize> {
    self.buffer.extend_from_slice(data);
    if self.buffer.len() >= ZIP_CHUNK_SIZE {
      self.send_buffered()?;
    }
    Ok(data.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

enum ZipSource {
  Directory { dir: PathBuf, root: PathBuf },
  Collection(Vec<(String, PathBuf)>),
}

pub fn stream_directory_zip(dir_path: PathBuf, root: PathBuf, allow_external: bool) -> Body {
  stream_zip(ZipSource::Directory { dir: dir_path, root }, allow_external)
}

pub fn stream_collection_zip(entries: Vec<(String, PathBuf)>, allow_external: bool) -> Body {
  stream_zip(ZipSource::Collection(entries), allow_external)
}

fn stream_zip(source: ZipSource, allow_external: bool) -> Body {
  let (tx, rx) = mpsc::channel::<ZipChunk>(8);

  tokio::task::spawn_blocking(move || {
    let error_tx = tx.clone();
    let sink = ChannelWriter { tx, buffer: Vec::with_capacity(ZIP_CHUNK_SIZE) };

    if let Err(error) = write_zip(sink, &source, allow_external) {
      if error.kind() != io::ErrorKind::BrokenPipe {
        eprintln!("[ZIP] Failed to build an archive: {}", error);
      }

      let _ = error_tx.blocking_send(Err(error));
    }
  });

  Body::from_stream(ReceiverStream::new(rx))
}

struct ZipContext<'a> {
  root: &'a Path,
  allow_external: bool,
}

fn write_zip(sink: ChannelWriter, source: &ZipSource, allow_external: bool) -> io::Result<()> {
  let mut zip = ZipWriter::new_stream(sink);

  match source {
    ZipSource::Directory { dir, root } => {
      let context = ZipContext { root, allow_external };
      let mut ancestors = vec![dir.clone()];
      add_directory(&mut zip, &context, dir, "", &mut ancestors)?;
    }
    ZipSource::Collection(entries) => {
      for (name, path) in entries {
        let Ok(metadata) = fs::metadata(path) else {
          continue;
        };

        if metadata.is_dir() {
          let context = ZipContext { root: path, allow_external };
          let child_prefix = format!("{name}/");
          start_directory(&mut zip, &child_prefix)?;

          let mut ancestors = vec![path.clone()];
          add_directory(&mut zip, &context, path, &child_prefix, &mut ancestors)?;
        } else {
          add_file(&mut zip, name.clone(), path, &metadata)?;
        }
      }
    }
  }

  let mut sink = zip.finish().map_err(io::Error::other)?.into_inner();
  sink.send_buffered()
}

fn start_directory<W: Write + Seek>(zip: &mut ZipWriter<W>, prefix: &str) -> io::Result<()> {
  let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Stored)
    .unix_permissions(0o755);
  zip.start_file(prefix, options)?;
  Ok(())
}

fn add_file<W: Write + Seek>(zip: &mut ZipWriter<W>, name: String, path: &Path, metadata: &fs::Metadata) -> io::Result<()> {
  let Ok(mut file) = File::open(path) else {
    return Ok(());
  };

  let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .compression_level(Some(3))
    .large_file(metadata.len() >= u32::MAX as u64);

  zip.start_file(name, options)?;
  io::copy(&mut file, zip)?;
  Ok(())
}

fn add_directory<W: Write + Seek>(
  zip: &mut ZipWriter<W>,
  context: &ZipContext,
  dir: &Path,
  prefix: &str,
  ancestors: &mut Vec<PathBuf>,
) -> io::Result<()> {
  let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)?.flatten().collect();
  entries.sort_by_key(|entry| entry.file_name());

  for entry in entries {
    let Some(resolved) = resolve_entry(context.root, &entry, context.allow_external) else {
      continue;
    };
    let name = format!("{prefix}{}", entry.file_name().to_string_lossy());

    if resolved.metadata.is_dir() {
      if ancestors.contains(&resolved.path) {
        continue;
      }

      let child_prefix = format!("{name}/");
      start_directory(zip, &child_prefix)?;

      ancestors.push(resolved.path.clone());
      add_directory(zip, context, &resolved.path, &child_prefix, ancestors)?;
      ancestors.pop();
    } else {
      add_file(zip, name, &resolved.path, &resolved.metadata)?;
    }
  }

  Ok(())
}
