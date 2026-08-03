use crate::settings::AppSettings;
use gtk4::gdk::RGBA;
use gtk4::gio;
use gtk4::prelude::*;
use pango::FontDescription;
use std::env;
use vte4::prelude::*;
use vte4::{Format, PtyFlags, Terminal};

/// Accent CSS classes cycling across tabs (green / blue / purple).
pub const ACCENT_CLASSES: [&str; 3] = ["green", "blue", "purple"];

pub fn create_terminal(settings: &AppSettings) -> Terminal {
    let terminal = Terminal::new();
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);
    terminal.set_scrollback_lines(10_000);
    terminal.set_mouse_autohide(true);
    terminal.set_allow_hyperlink(true);
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_clear_background(false);

    apply_font(&terminal, settings);
    apply_palette(&terminal, &settings.theme);
    spawn_shell(&terminal);
    terminal
}

pub fn apply_font(terminal: &Terminal, settings: &AppSettings) {
    let font = FontDescription::from_string(&settings.font_description());
    terminal.set_font(Some(&font));
}

pub fn apply_palette(terminal: &Terminal, theme: &str) {
    let (fg_hex, bg, palette_hex) = theme_colors(theme);
    let fg = parse_rgba(fg_hex);
    let bg = RGBA::new(bg.0, bg.1, bg.2, bg.3);

    let palette: Vec<RGBA> = palette_hex.iter().map(|h| parse_rgba(h)).collect();
    let refs: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &refs);
}

type RgbaF = (f32, f32, f32, f32);

fn theme_colors(theme: &str) -> (&'static str, RgbaF, [&'static str; 16]) {
    match theme {
        "nord" => (
            "#d8dee9",
            (0.180, 0.204, 0.251, 0.55), // #2e3440
            [
                "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0",
                "#e5e9f0", "#4c566a", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead",
                "#8fbcbb", "#eceff4",
            ],
        ),
        "solarized-dark" => (
            "#839496",
            (0.000, 0.169, 0.212, 0.55), // #002b36
            [
                "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198",
                "#eee8d5", "#002b36", "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4",
                "#93a1a1", "#fdf6e3",
            ],
        ),
        "light" => (
            "#1a1d23",
            (0.961, 0.965, 0.973, 0.85), // #f5f6f8
            [
                "#1a1d23", "#e35d6a", "#2f9e6e", "#b08900", "#3b82c4", "#8b6cc7", "#2a9d8f",
                "#e6e8eb", "#6b7280", "#ef7a84", "#3dd68c", "#e5c07b", "#6cb6ff", "#b794f6",
                "#56b6c2", "#ffffff",
            ],
        ),
        // glass-dark (default)
        _ => (
            "#e6e8eb",
            (0.051, 0.059, 0.071, 0.55), // #0d0f12
            [
                "#0d0f12", "#ff6b6b", "#3dd68c", "#e5c07b", "#6cb6ff", "#b794f6", "#56b6c2",
                "#e6e8eb", "#5c6370", "#ff8787", "#5eead4", "#f0d78c", "#89b4ff", "#c4b5fd",
                "#67e8f9", "#ffffff",
            ],
        ),
    }
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

/// Paste text into the PTY without a trailing newline (user confirms with Enter).
pub fn feed_text(terminal: &Terminal, text: &str) {
    terminal.feed_child(text.as_bytes());
}

/// Write bytes to the terminal display as if from the child (does not send to the PTY).
pub fn feed_output(terminal: &Terminal, text: &str) {
    terminal.feed(text.as_bytes());
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
