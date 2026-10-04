#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod api;
mod archive;
mod assets;
mod cli;
mod client;
mod config;
mod context_menu;
mod db;
mod host;
mod launcher;
mod paths;
mod raw;
mod resolve;
mod selection;
mod startup;
mod state;
mod token;
mod ui;

use std::{env, process};

use crate::{cli::Command, config::Config};

fn load_config(is_host: bool) -> Config {
  let (config, problem) = Config::load();

  if let (Some(reason), false) = (problem, is_host) {
    ui::warning(&format!("{reason}\n\nUsing the default settings for now."));
  }
  config
}

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();

  let command = match cli::parse(&args) {
    Ok(command) => command,
    Err(message) => {
      ui::error(&format!("{message}\n\n{}", cli::USAGE));
      process::exit(2);
    }
  };

  match command {
    Command::Help => ui::info(cli::USAGE),
    Command::Register => match context_menu::register() {
      Ok(_) => ui::info("Rockbreaker successfully registered its context submenu in the file explorer."),
      Err(e) => ui::error(&format!("Failed to register the context menu: {e}")),
    },
    Command::Unregister => match context_menu::unregister() {
      Ok(_) => ui::info("Rockbreaker successfully unregistered from the file explorer context menu."),
      Err(e) => ui::error(&format!("Failed to unregister the context menu: {e}")),
    },
    Command::Host => host::run(load_config(true)),
    command => {
      let config = load_config(false);

      if let Err(message) = launcher::run(command, config) {
        ui::error(&message);
        process::exit(1);
      }
    }
  }
}
