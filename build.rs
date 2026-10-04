use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use winresource::WindowsResource;

fn main() -> io::Result<()> {
  println!("cargo:rerun-if-changed=./assets/icon.ico");
  println!("cargo:rerun-if-changed=./web/src");
  println!("cargo:rerun-if-changed=./web/public");
  println!("cargo:rerun-if-changed=./web/index.html");
  println!("cargo:rerun-if-changed=./web/package.json");
  println!("cargo:rerun-if-changed=./web/vite.config.ts");

  if env::var_os("CARGO_CFG_WINDOWS").is_some() {
    let mut res = WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile()?;
  }

  let profile = env::var("PROFILE").unwrap_or_default();
  let dist_dir = Path::new("web/dist");

  if !dist_dir.exists() {
    let _ = fs::create_dir_all(dist_dir);
  }

  if profile == "release" || !dist_dir.join("index.html").exists() {
    println!("cargo:warning=[Rockbreaker] Compiling Vite frontend...");

    let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };

    if !Path::new("web/node_modules").exists() {
      let _ = Command::new(npm_cmd)
        .arg("install")
        .current_dir("web")
        .status();
    }

    let status = Command::new(npm_cmd)
      .args(["run", "build"])
      .current_dir("web")
      .status();

    match status {
      Ok(s) if s.success() => println!("cargo:warning=[Rockbreaker] Vite frontend built successfully."),
      Ok(s) => println!("cargo:warning=[Rockbreaker] Vite build failed with status: {}", s),
      Err(e) => println!("cargo:warning=[Rockbreaker] Could not run npm build: {}. Ensure Node.js is installed.", e),
    }
  }

  Ok(())
}
