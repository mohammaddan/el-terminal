mod app;
mod ask;
mod blur;
mod chrome;
mod context_menu;
mod env_context;
mod launch;
mod llm;
mod settings;
mod settings_ui;
mod tab_bar;
mod terminal_links;
mod terminal_tab;

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use launch::LaunchOptions;

const APP_ID: &str = "com.elterminal.app";

fn main() {
    // Independent windows so `el-terminal --working-directory=…` from the
    // file manager opens a fresh session instead of focusing an old one.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.add_main_option(
        "working-directory",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::Filename,
        "Directory to start in",
        Some("DIR"),
    );
    app.add_main_option(
        "dir",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::Filename,
        "Alias for --working-directory",
        Some("DIR"),
    );
    app.add_main_option(
        "command",
        glib::Char::from(b'e'),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        "Command to run instead of the interactive shell",
        Some("COMMAND"),
    );
    app.add_main_option(
        "new-window",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Open a new window (default; always opens a new window)",
        None,
    );

    app.connect_handle_local_options(|_, dict| {
        let mut opts = LaunchOptions::default();

        let dir_raw = lookup_filename_option(dict, "working-directory")
            .or_else(|| lookup_filename_option(dict, "dir"));
        if let Some(dir) = dir_raw {
            match launch::normalize_working_directory(&dir) {
                Ok(normalized) => opts.working_directory = Some(normalized),
                Err(err) => eprintln!("el-terminal: {err}"),
            }
        }

        if let Ok(Some(cmd)) = dict.lookup::<String>("command") {
            if !cmd.is_empty() {
                opts.command = Some(cmd);
            }
        }

        if dict.contains("new-window") {
            opts.new_window = true;
        }

        launch::store(opts);
        -1 // continue startup
    });

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("theme.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("display"),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(app::build_ui);
    app.run();
}

/// `G_OPTION_ARG_FILENAME` values are often null-terminated bytestrings.
fn lookup_filename_option(dict: &glib::VariantDict, key: &str) -> Option<String> {
    if let Ok(Some(dir)) = dict.lookup::<String>(key) {
        let dir = trim_c_string(dir);
        if !dir.is_empty() {
            return Some(dir);
        }
    }
    if let Ok(Some(bytes)) = dict.lookup::<Vec<u8>>(key) {
        let dir = trim_c_string(String::from_utf8_lossy(&bytes).into_owned());
        if !dir.is_empty() {
            return Some(dir);
        }
    }
    None
}

fn trim_c_string(s: String) -> String {
    s.trim_end_matches('\0').to_string()
}
