# Rockbreaker

Rockbreaker is a small file sharing tool that lives in your system tray. Right-click a file or folder, copy a link, and send it to anyone. They get a page with a preview and a download button, or a direct download if you prefer.

It's built to be used with [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/), so you can share files straight from your own computer without opening any ports. It also works with a forwarded port or other tunnelling services.

## Features

- **Share anything**: Both files and folders can be shared, no matter their size.
- **Background process**: It runs quietly in the background, using no CPU and less than 1 MB of memory while idle.
- **Persistence**: Shared items stay shared for as long as Rockbreaker is running, and it will remember shared items when closed and reopened. You can manually stop sharing items through the dashboard.
- **Dashboard**: It has a web UI for editing app settings and managing everything you are sharing. This allows you to see everything shared, edit their IDs, copy their links, and stop sharing.
- **Context menu integration** (Windows): There are settings to enable and customize a context menu option for files and folders in Windows Explorer. This allows you to simply right-click anything you want to share and copy a link to it.
- **Command line support**: For scripting and for use on platforms without a context menu integration, it has full support for use through its CLI.
- **Previews**: The share links for folders allow you to navigate the entire folder structure in your browser. Most common image, video, and audio file formats can be viewed/listened to directly in the browser. Markdown files can be viewed as rendered or in their raw text form.
- **Discord Embeds**: Links to shared files will show a rich embed in Discord. Images, videos, and small Markdown files will embed directly, allowing you to see them without leaving Discord.

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
- **Share** shares it without copying anything. It can be found in the dashboard.
- **Share and open** shares it and opens the link to the page in a browser, without copying the link.

Under Settings you can turn any of these off and put them in the order you want.

You can also share from the command line:

```
rockbreaker [options] <path>...
```

| Option | Description |
| --- | --- |
| `--id <id>` | Host the item at a specific ID instead of a random one (letters, numbers, `_` and `-`). |
| `--direct` | Copy the direct download link instead of the share page link. |
| `--no-copy` | Don't copy anything. |
| `--open` | Open the share page link in a browser instead of copying it. |

`--direct`, `--no-copy` and `--open` can't be combined.

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
| Config | Next to the executable (`rockbreaker.config.toml`) | `<config dir>/rockbreaker/rockbreaker.config.toml` |
| Database | Next to the executable (`rockbreaker.db`) | `<local data dir>/rockbreaker/rockbreaker.db` |

Settings like the URL base, the context menu, run on startup, and ignored files are managed from the dashboard's Settings tab rather than the config file.

### Ignoring files

Files and folders found *inside* of a shared folder can be left out. Under Settings in the dashboard there's a global ignore list (written like the lines of a `.gitignore` file, so `node_modules`, `*.log`, `/build` and `!keep.log` all work) and a switch for following the `.gitignore` files in the shared folder, which is on by default. Only the `.gitignore` files from the shared folder down are looked at, never the ones above it.

The host enforces this, not the web page: ignored entries aren't listed, can't be opened or downloaded by guessing their URL, and are left out of ZIPs. Something that is shared directly is always shared, even if it matches an ignore pattern.

## Security

Please read this before exposing Rockbreaker to the internet.

**The host token.** On first run, Rockbreaker generates a host token. It authenticates you as the host and stops other people from using your Rockbreaker API. **Anyone who has this token can access any file on your computer that Rockbreaker can read, for as long as Rockbreaker is running.** Treat it like a password, and don't share dashboard links that contain it.

If the token is ever leaked, open **Settings** in the dashboard and click **Generate new token**. The old token stops working immediately.

**Symlinks.** By default, Rockbreaker refuses to serve shared symlinks that point outside of what you've shared. You can allow this with `allow_external_symlinks` in the config file, but doing so can expose files you never meant to share, so it's best left off.

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

### Adding a file preview

The page for a shared file previews it with the first *viewer* that accepts it. Viewers live in `web/src/viewers/modules/`, one folder each, and are picked up automatically. To add one, create `modules/<name>/index.ts` that default-exports `defineViewer({ id, test, load })` and a Vue component for it. `test` gets the file's name, extension and size, and `readSlice(start, end)` to look at its bytes (for magic numbers and so on). `load` imports the component, which gets the file as its `file` prop. A viewer can also provide `metadata(file)`, which returns extra details (like the dimensions of an image) to add to the line under the file's name. Viewers are tried from the highest `priority` to the lowest.

## Platform support

The core sharing functionality is cross-platform. The file explorer context menu and run on startup features currently only work on Windows, and are disabled in Settings elsewhere. On other platforms you can share using the command line.

## Why "Rockbreaker"?

Rockbreaker is the successor in spirit to Snapmaw, an earlier file sharing tool built on top of [croc](https://github.com/schollz/croc). Both are named after machines from the *Horizon* games. The Snapmaw is modelled after a crocodile, and the Rockbreaker is a machine that tunnels underground, which fits a tool built around tunnels.
