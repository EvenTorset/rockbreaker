use std::{
  io::Cursor,
  net::SocketAddr,
  process,
  sync::{Arc, Mutex},
};

use axum::{
  routing::{get, post},
  Router, ServiceExt,
};
use tao::{
  event::{Event, StartCause},
  event_loop::{ControlFlow, EventLoopBuilder},
};
use tower_http::normalize_path::{NormalizePath, NormalizePathLayer};
use tower_layer::Layer;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

use crate::{
  api::{
    api_dashboard_handler, api_item_root_detail_handler, api_item_subpath_detail_handler, api_regenerate_token_handler,
    api_register_handler, api_rename_handler, api_settings_handler, api_stop_handler, api_update_settings_handler,
  },
  assets::spa_fallback_handler,
  config::Config,
  db::Database,
  paths::db_file_path,
  raw::{raw_item_root_handler, raw_item_subpath_handler},
  selection::SelectionBatcher,
  state::AppState,
  ui,
};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../assets/icon.ico");
const TRAY_ICON_SIZE: u32 = 32;

enum UserEvent {
  Menu(MenuEvent),
  Tray(TrayIconEvent),
  ServerStopped(String),
}

fn build_app(state: AppState) -> NormalizePath<Router> {
  let router = Router::new()
    .route("/!/api/dashboard", get(api_dashboard_handler))
    .route("/!/api/settings", get(api_settings_handler).post(api_update_settings_handler))
    .route("/!/api/token/regenerate", post(api_regenerate_token_handler))
    .route("/!/api/item/:host_id", get(api_item_root_detail_handler))
    .route("/!/api/item/:host_id/*subpath", get(api_item_subpath_detail_handler))
    .route("/!/api/register", post(api_register_handler))
    .route("/!/api/stop/:host_id", post(api_stop_handler))
    .route("/!/api/rename/:host_id", post(api_rename_handler))
    .route("/!/raw/:host_id", get(raw_item_root_handler))
    .route("/!/raw/:host_id/*subpath", get(raw_item_subpath_handler))
    .fallback(spa_fallback_handler)
    .with_state(state);

  NormalizePathLayer::trim_trailing_slash().layer(router)
}

fn load_tray_icon() -> Result<Icon, String> {
  let icons = ico::IconDir::read(Cursor::new(TRAY_ICON_BYTES)).map_err(|e| e.to_string())?;
  let entry = icons
    .entries()
    .iter()
    .min_by_key(|entry| entry.width().abs_diff(TRAY_ICON_SIZE))
    .ok_or("the icon file has no images")?;
  let image = entry.decode().map_err(|e| e.to_string())?;

  Icon::from_rgba(image.rgba_data().to_vec(), image.width(), image.height()).map_err(|e| e.to_string())
}

fn build_tray(menu: Menu) -> Result<TrayIcon, String> {
  TrayIconBuilder::new()
    .with_icon(load_tray_icon()?)
    .with_tooltip("Rockbreaker")
    .with_menu(Box::new(menu))
    .with_menu_on_left_click(false)
    .build()
    .map_err(|e| e.to_string())
}

pub fn run(config: Config) -> ! {
  let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
    Ok(runtime) => runtime,
    Err(error) => {
      eprintln!("Could not start the async runtime: {}", error);
      process::exit(1);
    }
  };

  let listener = match runtime.block_on(tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))) {
    Ok(listener) => listener,
    Err(error) => {
      eprintln!("Failed to bind to port {}: {}", config.port, error);
      process::exit(1);
    }
  };

  let db = match Database::open(&db_file_path()) {
    Ok(db) => Arc::new(db),
    Err(message) => {
      ui::error(&message);
      process::exit(1);
    }
  };

  match db.items() {
    Ok(items) => println!("[HOST] Restored {} hosted item(s).", items.len()),
    Err(error) => eprintln!("[HOST] Could not read the hosted items: {}", error),
  }
  if config.allow_external_symlinks {
    println!("Warning: allow_external_symlinks is on. Symlinks may expose files outside of shared directories.");
  }

  let app = build_app(AppState {
    config: config.clone(),
    db: db.clone(),
    selections: Arc::new(SelectionBatcher::default()),
  });

  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

  let proxy = Mutex::new(event_loop.create_proxy());
  TrayIconEvent::set_event_handler(Some(move |event| {
    let _ = proxy.lock().unwrap().send_event(UserEvent::Tray(event));
  }));
  let proxy = Mutex::new(event_loop.create_proxy());
  MenuEvent::set_event_handler(Some(move |event| {
    let _ = proxy.lock().unwrap().send_event(UserEvent::Menu(event));
  }));

  let server_proxy = event_loop.create_proxy();
  runtime.spawn(async move {
    let server = axum::serve(
      listener,
      ServiceExt::<axum::extract::Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    );

    let message = match server.await {
      Ok(()) => "The Rockbreaker server stopped.".to_string(),
      Err(error) => format!("The Rockbreaker server stopped: {error}"),
    };
    let _ = server_proxy.send_event(UserEvent::ServerStopped(message));
  });

  let menu = Menu::new();
  let open_item = MenuItem::new("Open Dashboard", true, None);
  let quit_item = MenuItem::new("Quit Rockbreaker", true, None);
  let open_id = open_item.id().clone();
  let quit_id = quit_item.id().clone();
  let menu_result = menu.append_items(&[&open_item, &PredefinedMenuItem::separator(), &quit_item]);

  let open_dashboard = move || {
    let dashboard_url = config.dashboard_url(&db.token().unwrap_or_default());
    if let Err(error) = open::that_detached(&dashboard_url) {
      ui::error(&format!("Could not open the dashboard in a browser: {error}\n\nDashboard: {dashboard_url}"));
    }
  };

  let mut menu = Some(menu);
  let mut tray_icon: Option<TrayIcon> = None;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    let mut quit = || {
      tray_icon = None;
      *control_flow = ControlFlow::Exit;
    };

    match event {
      Event::NewEvents(StartCause::Init) => {
        let created = menu_result
          .as_ref()
          .map_err(|e| e.to_string())
          .and_then(|_| build_tray(menu.take().expect("the tray is only created once")));

        match created {
          Ok(tray) => tray_icon = Some(tray),
          Err(error) => {
            ui::error(&format!("Could not create the tray icon: {error}\n\nRockbreaker needs a system tray to run in the background."));
            quit();
          }
        }
      }
      Event::UserEvent(UserEvent::Menu(event)) if event.id == open_id => open_dashboard(),
      Event::UserEvent(UserEvent::Menu(event)) if event.id == quit_id => quit(),
      Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      })) => open_dashboard(),
      Event::UserEvent(UserEvent::ServerStopped(message)) => {
        ui::error(&message);
        quit();
      }
      _ => {}
    }
  })
}
