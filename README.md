# Rockbreaker

Rockbreaker is a small file sharing tool that lives in your system tray. Right-click a file or folder, copy a link, and send it to anyone. They get a page with a preview and a download button, or a direct download if you prefer.

It's built to be used with [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/), so you can share files straight from your own computer without opening any ports. It also works with a forwarded port or other tunnelling services.

## Features

- **Share anything**: single files, folders, or a multi-selection that is shared together as one item.
- **Two kinds of links**: a *share link* to a web page with a preview and download button, or a *direct download link* that starts downloading right away.
- **Context menu integration** (Windows): adds "Share with Rockbreaker..." to the right-click menu of files and folders.
- **Dashboard**: a web UI for managing everything you're sharing. Rename an item's ID, copy its links, or stop sharing it.
- **Runs quietly in the background** with a system tray icon, and can optionally start when you log in (Windows).
- **Command line support** for scripting and for use on platforms without a context menu integration.

## Getting started

1. Run Rockbreaker once. This generates the default config file and opens the dashboard.
2. In the dashboard, open the **Settings** tab and set the **URL base** (the public address your tunnel or forwarded port is reachable at, e.g. `https://tunnel.example.com`) along with any other settings you want.
3. Open the generated `rockbreaker.config.toml` in a text editor and set `port` to the port you want Rockbreaker to listen on. This should match whatever your tunnel or port forward points to.
4. Quit Rockbreaker by right-clicking the tray icon and choosing **Quit Rockbreaker**.
5. Start it again so it picks up the modified config.

You're now ready to share files.

### Sharing files

If you enabled the context menu in Settings, right-click any file or folder (or a selection of several) and choose **Share with Rockbreaker...**:

- **Copy share link** copies a link to a page where the recipient can preview and download what you shared.
- **Copy direct download link** copies a link that downloads immediately, skipping the page.

You can also share from the command line:

```
rockbreaker [options] <path>...
```

| Option | Description |
| --- | --- |
| `--id <id>` | Host the item at a specific ID instead of a random one (letters, numbers, `_` and `-`). |
| `--direct` | Copy the direct download link instead of the share page link. |

Other commands:

| Command | Description |
| --- | --- |
| `rockbreaker` | Start Rockbreaker in the background if it isn't running, then open the dashboard. |
| `rockbreaker --background` | Start in the background without opening the dashboard. |
| `rockbreaker --register` | Add Rockbreaker to the file explorer context menu (Windows). |
| `rockbreaker --unregister` | Remove it from the context menu again. |
| `rockbreaker --help` | Show usage. |

### Managing what you share

Run Rockbreaker again, or click its tray icon, to open the dashboard. Everything you're currently sharing is listed under hosted items. From there you can change an item's ID, copy its links, or stop sharing it.

## Configuration

`rockbreaker.config.toml` is created on first run. If it can't be parsed, Rockbreaker warns you and falls back to the defaults.

| Key | Default | Description |
| --- | --- | --- |
| `port` | `8080` | The port Rockbreaker listens on. |
| `allow_external_symlinks` | `false` | Allow shared symlinks to point at parts of your system that aren't being shared. Leaving this off is strongly recommended. |

Where the files live:

|  | Windows | Linux / macOS |
| --- | --- | --- |
| Config | Next to the executable | `<config dir>/rockbreaker/rockbreaker.config.toml` |
| Database | Next to the executable (`rockbreaker.db`) | `<local data dir>/rockbreaker/rockbreaker.db` |

Settings like the URL base, the context menu, and run on startup are managed from the dashboard's Settings tab rather than the config file.

## Security

Please read this before exposing Rockbreaker to the internet.

**The host token.** On first run, Rockbreaker generates a host token. It authenticates you as the host and stops other people from using your Rockbreaker API. **Anyone who has this token can access any file on your computer that Rockbreaker can read, for as long as Rockbreaker is running.** Treat it like a password, and don't share dashboard links that contain it.

If the token is ever leaked, open **Settings** in the dashboard and click **Generate new token**. The old token stops working immediately.

**Symlinks.** By default, Rockbreaker refuses to serve shared symlinks that point outside of what you've shared. You can allow this with `allow_external_symlinks` in the config file, but doing so can expose files you never meant to share, so it's best left off.

**Network exposure.** Rockbreaker listens on all network interfaces on the configured port. If you use a tunnel, make sure you understand what it exposes, and consider firewalling the port from your local network if you don't want it reachable there.

## Building from source

You'll need:

- [Rust](https://rustup.rs/) 1.88 or newer
- [Node.js](https://nodejs.org/) and npm (the dashboard is a Vue app that gets built and embedded into the binary)
- On Linux, GTK 3 development libraries (used for the tray icon and dialogs)

```
cargo build --release
```

The build script runs `npm install` and `npm run build` in `web/` automatically for release builds, and for debug builds when `web/dist` doesn't exist yet.

### Working on the web UI

```
cd web
npm install
npm run dev
```

To be able to use the host functions in the dev server-hosted UI, you need to get the token from the database and pass it to the dashboard through the `token` query parameter, e.g. `http://localhost:5173/?token=abcDEF0123...`

## Platform support

The core sharing functionality is cross-platform. The file explorer context menu and run on startup features currently only work on Windows, and are disabled in Settings elsewhere. On other platforms you can share using the command line.

## Why "Rockbreaker"?

Rockbreaker is the successor in spirit to Snapmaw, an earlier file sharing tool built on top of [croc](https://github.com/schollz/croc). Both are named after machines from the *Horizon* games. The Snapmaw is modelled after a crocodile, and the Rockbreaker is a machine that tunnels underground, which fits a tool built around tunnels.
