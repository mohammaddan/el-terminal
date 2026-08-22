use crate::settings::{AppSettings, ThemeStyle};
use crate::terminal_links;
use gtk4::gdk::RGBA;
use gtk4::gio;
use gtk4::prelude::*;
use pango::FontDescription;
use std::env;
use vte4::prelude::*;
use vte4::{Format, PtyFlags, Terminal};

/// Accent CSS classes cycling across tabs (green / blue / purple).
pub const ACCENT_CLASSES: [&str; 3] = ["green", "blue", "purple"];

/// How to start the PTY child for a new terminal pane.
#[derive(Clone, Debug, Default)]
pub struct SpawnOpts {
    /// Override cwd; when `None`, uses the process current directory.
    pub working_directory: Option<String>,
    /// When set, run `$SHELL -c <command>` instead of an interactive shell.
    pub command: Option<String>,
}

pub fn create_terminal(settings: &AppSettings) -> Terminal {
    create_terminal_with(settings, SpawnOpts::default())
}

pub fn create_terminal_with(settings: &AppSettings, spawn: SpawnOpts) -> Terminal {
    let terminal = Terminal::new();
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);
    terminal.set_size_request(0, 0);
    terminal.set_scrollback_lines(10_000);
    terminal.set_mouse_autohide(true);
    terminal_links::setup(&terminal, &settings.style.accent_blue);
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_clear_background(false);

    apply_font(&terminal, settings);
    apply_palette(&terminal, &settings.style);
    spawn_shell(&terminal, &spawn);
    terminal
}

pub fn apply_font(terminal: &Terminal, settings: &AppSettings) {
    let font = FontDescription::from_string(&settings.font_description());
    terminal.set_font(Some(&font));
}

pub fn apply_palette(terminal: &Terminal, style: &ThemeStyle) {
    let fg = parse_rgba(&style.terminal_fg);
    let bg = RGBA::new(
        style.terminal_bg[0],
        style.terminal_bg[1],
        style.terminal_bg[2],
        style.terminal_bg[3],
    );

    let palette: Vec<RGBA> = style.palette.iter().map(|h| parse_rgba(h)).collect();
    let refs: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &refs);
}

fn parse_rgba(hex: &str) -> RGBA {
    RGBA::parse(hex).unwrap_or_else(|_| RGBA::new(1.0, 1.0, 1.0, 1.0))
}

pub fn spawn_shell(terminal: &Terminal, spawn: &SpawnOpts) {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    let cwd = spawn
        .working_directory
        .clone()
        .or_else(|| {
            env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
        });

    let argv_owned: Vec<String> = match &spawn.command {
        Some(cmd) => vec![shell, "-c".into(), cmd.clone()],
        None => vec![shell],
    };
    let argv: Vec<&str> = argv_owned.iter().map(String::as_str).collect();

    // spawn_async already watches the child and emits `child-exited`; do not
    // also call watch_child — that double-reaps and triggers GLib waitid warnings.
    terminal.spawn_async(
        PtyFlags::DEFAULT,
        cwd.as_deref(),
        &argv,
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        move |result| {
            if let Err(err) = result {
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
/// URLs are styled with the theme link color and OSC 8 hyperlinks for click-to-open.
pub fn feed_output(terminal: &Terminal, text: &str) {
    let styled = terminal_links::linkify(text, &terminal_links::link_color_for(terminal));
    terminal.feed(styled.as_bytes());
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
