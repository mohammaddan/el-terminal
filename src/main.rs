mod app;
mod ask;
mod blur;
mod chrome;
mod context_menu;
mod env_context;
mod llm;
mod settings;
mod settings_ui;
mod tab_bar;
mod terminal_tab;

use gtk4::prelude::*;
use gtk4::{Application, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

const APP_ID: &str = "com.terminalemulator.app";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

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
