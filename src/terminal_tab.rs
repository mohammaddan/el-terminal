use gtk4::gdk::RGBA;
use gtk4::gio;
use gtk4::prelude::*;
use pango::FontDescription;
use std::env;
use vte4::prelude::*;
use vte4::{Format, PtyFlags, Terminal};

/// Accent CSS classes cycling across tabs (green / blue / purple).
pub const ACCENT_CLASSES: [&str; 3] = ["green", "blue", "purple"];

/// Shared glass background (matches chrome fill).
const GLASS_BG: (f32, f32, f32, f32) = (0.051, 0.059, 0.071, 0.55);

pub fn create_terminal() -> Terminal {
    let terminal = Terminal::new();
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);
    terminal.set_scrollback_lines(10_000);
    terminal.set_mouse_autohide(true);
    terminal.set_allow_hyperlink(true);
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_clear_background(false);

    let font = FontDescription::from_string(
        "JetBrains Mono 11, Fira Code 11, Cascadia Code 11, monospace 11",
    );
    terminal.set_font(Some(&font));

    apply_palette(&terminal);
    spawn_shell(&terminal);
    terminal
}

pub fn apply_palette(terminal: &Terminal) {
    let bg = RGBA::new(GLASS_BG.0, GLASS_BG.1, GLASS_BG.2, GLASS_BG.3);
    let fg = parse_rgba("#e6e8eb");

    let palette = [
        parse_rgba("#0d0f12"), // 0 black
        parse_rgba("#ff6b6b"), // 1 red
        parse_rgba("#3dd68c"), // 2 green
        parse_rgba("#e5c07b"), // 3 yellow
        parse_rgba("#6cb6ff"), // 4 blue
        parse_rgba("#b794f6"), // 5 magenta
        parse_rgba("#56b6c2"), // 6 cyan
        parse_rgba("#e6e8eb"), // 7 white
        parse_rgba("#5c6370"), // 8 bright black
        parse_rgba("#ff8787"), // 9 bright red
        parse_rgba("#5eead4"), // 10 bright green
        parse_rgba("#f0d78c"), // 11 bright yellow
        parse_rgba("#89b4ff"), // 12 bright blue
        parse_rgba("#c4b5fd"), // 13 bright magenta
        parse_rgba("#67e8f9"), // 14 bright cyan
        parse_rgba("#ffffff"), // 15 bright white
    ];
    let refs: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &refs);
}

fn parse_rgba(hex: &str) -> RGBA {
    RGBA::parse(hex).unwrap_or_else(|_| RGBA::new(1.0, 1.0, 1.0, 1.0))
}

pub fn spawn_shell(terminal: &Terminal) {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    let argv = [shell.as_str()];
    let cwd = env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()));

    let term = terminal.clone();
    terminal.spawn_async(
        PtyFlags::DEFAULT,
        cwd.as_deref(),
        &argv,
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok(pid) => {
                term.watch_child(pid);
            }
            Err(err) => {
                eprintln!("failed to spawn shell: {err}");
            }
        },
    );
}

pub fn copy_selection(terminal: &Terminal) {
    if terminal.has_selection() {
        terminal.copy_clipboard_format(Format::Text);
    }
}

pub fn paste_clipboard(terminal: &Terminal) {
    terminal.paste_clipboard();
}

pub fn select_all(terminal: &Terminal) {
    terminal.select_all();
}

pub fn default_title() -> String {
    env::current_dir()
        .ok()
        .map(|p| shorten_path(&p))
        .unwrap_or_else(|| "terminal".into())
}

pub fn title_from_terminal(terminal: &Terminal) -> String {
    terminal
        .window_title()
        .map(|t| t.to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(default_title)
}

fn shorten_path(path: &std::path::Path) -> String {
    if let Some(home) = env::var_os("HOME") {
        if let Ok(stripped) = path.strip_prefix(&home) {
            if stripped.as_os_str().is_empty() {
                return "~".into();
            }
            return format!("~/{}", stripped.display());
        }
    }
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
