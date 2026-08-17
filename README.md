![Screenshot](image.png)

# El-Terminal

Lightweight borderless terminal emulator for Linux (GTK4 + VTE), with optional OpenAI-compatible Ask AI.

## Features

**Window & chrome**
- Undecorated window with rounded translucent chrome (no OS titlebar)
- Compositor backdrop blur when available (`ext-background-effect-v1`, or KWin blur)
- Drag the window by grabbing empty space on the top bar
- Pill-shaped tabs with accent status dots

**Terminal**
- Multi-tab sessions (each pane has its own PTY / `$SHELL`)
- Split panes within a tab (resizable; nestable)
- Right-click context menu: Copy, Paste, Select All, New Tab, Previous/Next Tab, Split Right/Down, Close Pane, Focus Left/Right/Up/Down
- Ctrl+scroll to zoom font size (persisted)

**Settings** (`Ctrl+Shift+,` or menu → Settings…)
- Theme presets: Glass Dark, Nord, Solarized Dark, Light
- Font family and size
- OpenAI-compatible LLM: endpoint, API key, model
- In-shell Ask prefix (default `??`)
- Opt-in **Share terminal context with Ask AI** (off by default)
- Stored at `~/.config/el-terminal/settings.json`

**Ask AI**
- Side panel (toolbar **Ask** or `Ctrl+Shift+/`): prompt → reply; **Apply** / **Run** only on shell code blocks
- In-shell: type `?? your question` (or your configured prefix) and press Enter — the answer prints in the terminal below the command (panel stays closed)
- Terminal context (cwd, title, recent output) is sent only when **Share terminal context** is enabled; the Ask panel / in-shell output shows when sharing is active
- Streaming and non-streaming OpenAI-compatible `/chat/completions` responses
- Fully usable as a terminal when Ask is unconfigured or disabled

![Screenshot](image2.png)

## Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+C` / `V` | Copy / Paste |
| `Ctrl+Shift+A` | Select all |
| `Ctrl+Shift+T` | New tab |
| `Ctrl+PageUp` / `PageDown` | Previous / next tab |
| `Ctrl+Shift+W` | Close pane (or tab if last pane) |
| `Ctrl+Shift+R` / `D` | Split right / down |
| `Alt+Left` / `Right` / `Up` / `Down` | Focus pane in that direction |
| `Ctrl+Shift+,` | Settings |
| `Ctrl+Shift+/` | Toggle Ask panel |
| `Ctrl+Shift+Q` | Quit |
| `Ctrl+scroll` | Font size zoom |
| `?? …` + Enter | In-shell Ask (prefix configurable) |

![Screenshot](image3.png)

## Dependencies

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libvte-2.91-gtk4-dev
```

Rust toolchain: https://rustup.rs (`cargo` + `rustc`).

If `libvte-2.91-gtk4-dev` is not installed system-wide, this repo can use a local copy under `.local-deps/` (picked up automatically by `build.rs` when present).

## Build & run

```bash
cargo run --release
```

If VTE was linked from `.local-deps/`:

```bash
export LD_LIBRARY_PATH="$PWD/.local-deps/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
cargo run --release
```

## CLI

```bash
el-terminal
el-terminal --working-directory=/path/to/dir
el-terminal --dir=/path/to/dir              # alias
el-terminal --new-window                    # accepted; every launch is a new window
el-terminal --command 'htop'                # or -e 'htop'
el-terminal --working-directory ~/src --command 'ls -la'
```

Paths with spaces or Unicode work as normal arguments. For directories whose names start with `-`, use the equals form: `--working-directory=/-odd-name`.

`--command` / `-e` runs `$SHELL -c …` in the first pane (window closes when it exits). The working directory is passed to VTE’s spawn API (not via `bash -c "cd …"`). New tabs inherit the same folder because the process cwd is updated.

## Desktop integration (Ubuntu 24.04)

### Packages

```bash
cargo build --release
cargo deb                          # el-terminal
cargo deb --variant=nautilus       # el-terminal-nautilus (Nautilus menu)
sudo apt install ./target/debian/el-terminal_*.deb
sudo apt install ./target/debian/el-terminal-nautilus_*.deb
```

`el-terminal` **Recommends** `el-terminal-nautilus`. The Nautilus package depends on `python3-nautilus`.

### Default terminal

The `.deb` registers `/usr/bin/el-terminal` with `update-alternatives` for `x-terminal-emulator` (priority 50; does not force the selection or touch `gnome-terminal`).

```bash
sudo update-alternatives --config x-terminal-emulator
```

GNOME preferred terminal (used by some launchers):

```bash
gsettings set org.gnome.desktop.default-applications.terminal exec 'el-terminal'
gsettings set org.gnome.desktop.default-applications.terminal exec-arg '-e'
```

`xdg-terminal-exec` (optional, universe): prefer El-Terminal by putting `el-terminal.desktop` first in `~/.config/xdg-terminals.list`. See `packaging/xdg-terminals.list.example` (also installed under `/usr/share/doc/el-terminal/`).

### Open in El-Terminal (Nautilus)

After installing `el-terminal-nautilus`:

```bash
nautilus -q   # restart Files
```

Right-click a folder or the folder background → **Open in El-Terminal**. That runs:

```text
el-terminal --working-directory "/absolute/path"
```

as a direct argv list (safe for spaces, Unicode, and shell-special characters).

### Smoke test

```bash
./packaging/test-desktop-integration.sh
```

## Ask AI setup

1. Open **Settings** and set:
   - **Endpoint** — API base URL (e.g. `https://api.openai.com/v1` or `http://localhost:11434/v1` for Ollama)
   - **API key** — bearer token (use any non-empty value if your local server ignores auth)
   - **Model** — e.g. `gpt-4o-mini` or an Ollama model name
   - Optionally enable **Share terminal context with Ask AI**
2. Ask from the side panel, or in the shell:

```text
?? how do I find large files under the current directory
```


Optional checks before tagging:

```bash
cargo build --release
cargo deb
cargo deb --variant=nautilus
```

Inspect or edit the release after the workflow finishes:

```bash
gh release list
gh release view v0.4.0
gh release upload v0.4.0 target/debian/*.deb --clobber   # e.g. add nautilus .deb built locally
```

## Layout

```
src/
  main.rs          # app entry + CSS
  app.rs           # window, tabs, drag, shortcuts, settings / Ask wiring
  ask.rs           # Ask side panel + in-shell Ask
  env_context.rs   # terminal / shell / DB environment for Ask pre-prompt
  llm.rs           # OpenAI-compatible chat client (stream + non-stream)
  settings.rs      # persisted settings model
  settings_ui.rs   # Settings dialog
  blur.rs          # Wayland compositor backdrop blur
  tab_bar.rs       # pill tabs / chrome widgets
  terminal_tab.rs  # VTE session + palette + font
  context_menu.rs  # right-click menu + actions
  chrome.rs        # rounded frosted fill + grain
  theme.css        # colors and chrome
```
