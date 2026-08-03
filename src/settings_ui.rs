use crate::settings::{AppSettings, FONT_SIZE_MAX, FONT_SIZE_MIN, THEME_IDS, THEME_LABELS};
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Dialog, DropDown, Entry, Label, Orientation, PasswordEntry,
    SpinButton, Window,
};

/// Open the Settings dialog. On Save, updates `settings` via `on_save`.
pub fn open_settings_dialog(
    parent: &impl IsA<Window>,
    current: &AppSettings,
    on_save: impl Fn(AppSettings) + 'static,
) {
    let dialog = Dialog::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .default_width(420)
        .default_height(440)
        .build();
    dialog.add_css_class("settings-dialog");

    let content = GtkBox::new(Orientation::Vertical, 14);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.set_hexpand(true);

    // Appearance
    content.append(&section_label("Appearance"));

    let theme_row = labeled_row("Theme");
    let theme_strings: Vec<&str> = THEME_LABELS.to_vec();
    let theme_dropdown = DropDown::from_strings(&theme_strings);
    let theme_idx = THEME_IDS
        .iter()
        .position(|id| *id == current.theme.as_str())
        .unwrap_or(0) as u32;
    theme_dropdown.set_selected(theme_idx);
    theme_dropdown.set_hexpand(true);
    theme_row.append(&theme_dropdown);
    content.append(&theme_row);

    let font_row = labeled_row("Font family");
    let font_entry = Entry::new();
    font_entry.set_text(&current.font_family);
    font_entry.set_hexpand(true);
    font_entry.set_placeholder_text(Some("JetBrains Mono"));
    font_row.append(&font_entry);
    content.append(&font_row);

    let size_row = labeled_row("Font size");
    let size_spin = SpinButton::with_range(FONT_SIZE_MIN as f64, FONT_SIZE_MAX as f64, 1.0);
    size_spin.set_value(current.font_size as f64);
    size_spin.set_digits(0);
    size_spin.set_hexpand(true);
    size_row.append(&size_spin);
    content.append(&size_row);

    // LLM
    content.append(&section_label("LLM (OpenAI-compatible)"));

    let endpoint_row = labeled_row("Endpoint");
    let endpoint_entry = Entry::new();
    endpoint_entry.set_text(&current.llm_endpoint);
    endpoint_entry.set_hexpand(true);
    endpoint_entry.set_placeholder_text(Some("https://api.openai.com/v1"));
    endpoint_row.append(&endpoint_entry);
    content.append(&endpoint_row);

    let key_row = labeled_row("API key");
    let key_entry = PasswordEntry::new();
    key_entry.set_text(&current.llm_api_key);
    key_entry.set_hexpand(true);
    key_entry.set_show_peek_icon(true);
    key_row.append(&key_entry);
    content.append(&key_row);

    let model_row = labeled_row("Model");
    let model_entry = Entry::new();
    model_entry.set_text(&current.llm_model);
    model_entry.set_hexpand(true);
    model_entry.set_placeholder_text(Some("gpt-4o-mini"));
    model_row.append(&model_entry);
    content.append(&model_row);

    let prefix_row = labeled_row("Ask prefix");
    let prefix_entry = Entry::new();
    prefix_entry.set_text(&current.ask_prefix);
    prefix_entry.set_hexpand(true);
    prefix_entry.set_placeholder_text(Some("??"));
    prefix_entry.set_tooltip_text(Some(
        "Type this prefix in the shell then Enter to Ask AI (default: ??)",
    ));
    prefix_row.append(&prefix_entry);
    content.append(&prefix_row);

    let hint = Label::new(Some(
        "Endpoint is the API base (…/v1). In the shell, type the Ask prefix (default ??) then your question and press Enter.",
    ));
    hint.add_css_class("settings-hint");
    hint.set_wrap(true);
    hint.set_xalign(0.0);
    content.append(&hint);

    // Buttons
    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_halign(Align::End);
    buttons.set_margin_top(8);

    let cancel = Button::with_label("Cancel");
    cancel.add_css_class("settings-btn");
    let save = Button::with_label("Save");
    save.add_css_class("settings-btn");
    save.add_css_class("settings-btn-primary");
    buttons.append(&cancel);
    buttons.append(&save);
    content.append(&buttons);

    dialog.set_child(Some(&content));

    cancel.connect_clicked(clone!(
        #[strong]
        dialog,
        move |_| {
            dialog.close();
        }
    ));

    save.connect_clicked(clone!(
        #[strong]
        dialog,
        #[strong]
        theme_dropdown,
        #[strong]
        font_entry,
        #[strong]
        size_spin,
        #[strong]
        endpoint_entry,
        #[strong]
        key_entry,
        #[strong]
        model_entry,
        #[strong]
        prefix_entry,
        move |_| {
            let idx = theme_dropdown.selected() as usize;
            let theme = THEME_IDS
                .get(idx)
                .copied()
                .unwrap_or("glass-dark")
                .to_string();
            let mut settings = AppSettings {
                theme,
                font_size: size_spin.value() as u32,
                font_family: font_entry.text().to_string(),
                llm_endpoint: endpoint_entry.text().to_string(),
                llm_api_key: key_entry.text().to_string(),
                llm_model: model_entry.text().to_string(),
                ask_prefix: prefix_entry.text().to_string(),
            };
            settings.normalize();
            on_save(settings);
            dialog.close();
        }
    ));

    dialog.present();
}

fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("settings-section");
    label.set_xalign(0.0);
    label.set_margin_top(4);
    label
}

fn labeled_row(title: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.set_hexpand(true);
    let label = Label::new(Some(title));
    label.set_width_chars(12);
    label.set_xalign(0.0);
    label.set_halign(Align::Start);
    label.add_css_class("settings-label");
    row.append(&label);
    row
}
