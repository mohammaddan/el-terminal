# Terminal Emulator

Lightweight borderless terminal emulator for Linux (GTK4 + VTE).

## Features

- Undecorated window with rounded translucent chrome (no OS titlebar)
- Compositor backdrop blur when available (`ext-background-effect-v1`, or KWin blur)
- Pill-shaped tabs with accent status dots
- Drag the window by grabbing empty space on the top bar
- Right-click context menu: Copy, Paste, Select All, New Tab, Close Tab, Split Right/Down, Close Pane
- Split panes within a tab (resizable; nestable)
- Multi-tab sessions (each pane has its own PTY / `$SHELL`)
- Shortcuts: `Ctrl+Shift+C/V` copy/paste, `Ctrl+Shift+T` new tab, `Ctrl+Shift+W` close pane, `Ctrl+Shift+R/D` split right/down, `Ctrl+Shift+Q` quit

## Dependencies

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libvte-2.91-gtk4-dev
```

Rust toolchain: https://rustup.rs (`cargo` + `rustc`).

If `libvte-2.91-gtk4-dev` is not installed system-wide, this repo can use a local copy under `.local-deps/` (used automatically by `build.rs` when present).

## Build & run

```bash
cargo run --release
```

If VTE was linked from `.local-deps/`:

```bash
export LD_LIBRARY_PATH="$PWD/.local-deps/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
cargo run --release
```

## Layout

```
src/
  main.rs          # app entry + CSS
  app.rs           # window, tabs, drag, shortcuts
  blur.rs          # Wayland compositor backdrop blur
  tab_bar.rs       # pill tabs / chrome widgets
  terminal_tab.rs  # VTE session + palette
  context_menu.rs  # right-click menu + window
  chrome.rs        # rounded frosted fill + grain
  theme.css        # mockup colors and chrome
```
