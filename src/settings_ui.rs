use crate::settings::{
    default_theme_id, theme_ids, theme_labels, AppSettings, FONT_SIZE_MAX, FONT_SIZE_MIN,
};
use gtk4::glib::{self, clone};
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CheckButton, Dialog, DropDown, Entry,
    EventControllerScroll, EventControllerScrollFlags, Label, ListScrollFlags, ListView,
    Orientation, PasswordEntry, SpinButton, Widget, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// Open the Settings dialog.
///
/// - `on_preview` applies settings live (theme changes) without writing disk.
/// - `on_save` persists and applies the final settings.
pub fn open_settings_dialog(
    parent: &impl IsA<Window>,
    current: &AppSettings,
    on_preview: impl Fn(AppSettings) + 'static,
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
    let theme_id_list = theme_ids();
    let theme_label_list = theme_labels();
    let theme_label_refs: Vec<&str> = theme_label_list.iter().map(|s| s.as_str()).collect();
    let theme_dropdown = DropDown::from_strings(&theme_label_refs);
    let theme_idx = theme_id_list
        .iter()
        .position(|id| id == &current.theme)
        .unwrap_or(0) as u32;
    theme_dropdown.set_selected(theme_idx);
    theme_dropdown.set_hexpand(true);
    wire_theme_dropdown_wheel(&theme_dropdown);
    wire_theme_dropdown_scroll_to_active(&theme_dropdown);
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

    let share_ctx = CheckButton::with_label("Share terminal context with Ask AI");
    share_ctx.set_active(current.ask_share_terminal_context);
    share_ctx.set_tooltip_text(Some(
        "When enabled, Ask sends working directory, tab title, and recent terminal output to the LLM. Off by default.",
    ));
    share_ctx.add_css_class("settings-check");
    content.append(&share_ctx);

    let hint = Label::new(Some(
        "Endpoint is the API base (…/v1). In the shell, type the Ask prefix (default ??) then your question and press Enter. Terminal context is never sent unless “Share terminal context” is enabled.",
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

    let previous = current.clone();
    let committed = Rc::new(Cell::new(false));
    let on_preview = Rc::new(on_preview);

    // Connect after set_selected so the initial value does not fire a preview.
    theme_dropdown.connect_selected_notify(clone!(
        #[strong]
        previous,
        #[strong]
        on_preview,
        move |dropdown| {
            let idx = dropdown.selected() as usize;
            let theme = theme_ids()
                .get(idx)
                .cloned()
                .unwrap_or_else(default_theme_id);
            let mut settings = previous.clone();
            settings.apply_theme_preset(&theme);
            on_preview(settings);
        }
    ));

    cancel.connect_clicked(clone!(
        #[strong]
        dialog,
        #[strong]
        previous,
        #[strong]
        on_preview,
        #[strong]
        committed,
        move |_| {
            if !committed.get() {
                on_preview(previous.clone());
            }
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
        #[strong]
        share_ctx,
        #[strong]
        previous,
        #[strong]
        committed,
        move |_| {
            let idx = theme_dropdown.selected() as usize;
            let theme = theme_ids()
                .get(idx)
                .cloned()
                .unwrap_or_else(default_theme_id);

            let mut settings = previous.clone();
            if theme != settings.theme {
                settings.apply_theme_preset(&theme);
            }
            settings.font_size = size_spin.value() as u32;
            settings.font_family = font_entry.text().to_string();
            settings.llm_endpoint = endpoint_entry.text().to_string();
            settings.llm_api_key = key_entry.text().to_string();
            settings.llm_model = model_entry.text().to_string();
            settings.ask_prefix = prefix_entry.text().to_string();
            settings.ask_share_terminal_context = share_ctx.is_active();
            settings.normalize();
            committed.set(true);
            on_save(settings);
            dialog.close();
        }
    ));

    dialog.connect_close_request(clone!(
        #[strong]
        previous,
        #[strong]
        on_preview,
        #[strong]
        committed,
        move |_| {
            if !committed.get() {
                on_preview(previous.clone());
            }
            glib::Propagation::Proceed
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

/// Mouse wheel over the closed theme dropdown cycles the selection (and live preview).
fn wire_theme_dropdown_wheel(dropdown: &DropDown) {
    let scroll = EventControllerScroll::new(
        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::DISCRETE,
    );
    scroll.connect_scroll(clone!(
        #[strong]
        dropdown,
        move |_, _dx, dy| {
            let n = dropdown.model().map(|m| m.n_items()).unwrap_or(0);
            if n == 0 {
                return glib::Propagation::Proceed;
            }
            let cur = dropdown.selected();
            if dy > 0.0 && cur + 1 < n {
                dropdown.set_selected(cur + 1);
                return glib::Propagation::Stop;
            }
            if dy < 0.0 && cur > 0 {
                dropdown.set_selected(cur - 1);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    ));
    dropdown.add_controller(scroll);
}

/// When the dropdown popover opens, scroll the list so the active theme is visible.
fn wire_theme_dropdown_scroll_to_active(dropdown: &DropDown) {
    let wired = Rc::new(Cell::new(false));
    let try_wire = clone!(
        #[strong]
        dropdown,
        #[strong]
        wired,
        move || {
            if wired.get() {
                return;
            }
            let Some(list) = find_descendant::<ListView>(&dropdown) else {
                return;
            };
            wired.set(true);
            list.connect_map(clone!(
                #[strong]
                dropdown,
                move |list| {
                    let pos = dropdown.selected();
                    let list = list.clone();
                    // Wait a frame so the list has a size before scrolling.
                    glib::idle_add_local_once(move || {
                        list.scroll_to(
                            pos,
                            ListScrollFlags::FOCUS | ListScrollFlags::SELECT,
                            None,
                        );
                    });
                }
            ));
        }
    );

    try_wire();
    if !wired.get() {
        dropdown.connect_realize(clone!(
            #[strong]
            try_wire,
            move |_| try_wire()
        ));
        glib::idle_add_local_once(move || try_wire());
    }
}

fn find_descendant<T: IsA<Widget>>(root: &impl IsA<Widget>) -> Option<T> {
    let mut stack = Vec::new();
    if let Some(child) = root.first_child() {
        stack.push(child);
    }
    while let Some(widget) = stack.pop() {
        if let Ok(typed) = widget.clone().downcast::<T>() {
            return Some(typed);
        }
        let mut child = widget.first_child();
        while let Some(c) = child {
            let next = c.next_sibling();
            stack.push(c);
            child = next;
        }
    }
    None
}
