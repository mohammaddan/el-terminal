use crate::env_context::TerminalContext;
use crate::llm::{self, AskReply, ReplySegment};
use crate::settings::AppSettings;
use crate::terminal_tab;
use gtk4::glib::{self, clone};
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Label, Orientation, PolicyType, ScrolledWindow, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use vte4::prelude::*;
use vte4::{Format, Terminal};

/// Side panel for AI Ask: prompt → reply with Apply/Run only on shell code blocks.
pub struct AskPanel {
    root: GtkBox,
    results: GtkBox,
    status: Label,
    prompt: TextView,
    ask_btn: Button,
    visible: RefCell<bool>,
    busy: RefCell<bool>,
}

impl AskPanel {
    pub fn build() -> Rc<Self> {
        let root = GtkBox::new(Orientation::Vertical, 10);
        root.add_css_class("ask-panel");
        root.set_size_request(300, -1);
        root.set_hexpand(false);
        root.set_vexpand(true);
        root.set_halign(Align::End);
        root.set_valign(Align::Fill);
        root.set_visible(false);

        let header = GtkBox::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("Ask"));
        title.add_css_class("ask-title");
        title.set_hexpand(true);
        title.set_xalign(0.0);
        let close = Button::from_icon_name("window-close-symbolic");
        close.add_css_class("ask-close");
        close.set_tooltip_text(Some("Close"));
        header.append(&title);
        header.append(&close);
        root.append(&header);

        let prompt = TextView::new();
        prompt.add_css_class("ask-prompt");
        prompt.set_wrap_mode(gtk4::WrapMode::WordChar);
        prompt.set_accepts_tab(false);
        prompt.set_monospace(true);
        prompt.set_size_request(-1, 90);

        let prompt_scroll = ScrolledWindow::new();
        prompt_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
        prompt_scroll.set_child(Some(&prompt));
        prompt_scroll.set_min_content_height(90);
        prompt_scroll.add_css_class("ask-prompt-scroll");
        root.append(&prompt_scroll);

        let ask_btn = Button::with_label("Ask");
        ask_btn.add_css_class("ask-submit");
        ask_btn.set_halign(Align::End);
        root.append(&ask_btn);

        let status = Label::new(None);
        status.add_css_class("ask-status");
        status.set_wrap(true);
        status.set_xalign(0.0);
        root.append(&status);

        let results = GtkBox::new(Orientation::Vertical, 8);
        results.add_css_class("ask-results");
        results.set_hexpand(true);

        let results_scroll = ScrolledWindow::new();
        results_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        results_scroll.set_vexpand(true);
        results_scroll.set_child(Some(&results));
        root.append(&results_scroll);

        let panel = Rc::new(Self {
            root,
            results,
            status,
            prompt,
            ask_btn,
            visible: RefCell::new(false),
            busy: RefCell::new(false),
        });

        close.connect_clicked(clone!(
            #[strong]
            panel,
            move |_| {
                panel.set_visible(false);
            }
        ));

        panel
    }

    pub fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    pub fn set_visible(&self, visible: bool) {
        *self.visible.borrow_mut() = visible;
        self.root.set_visible(visible);
        if visible {
            self.prompt.grab_focus();
        }
    }

    pub fn toggle(&self) {
        self.set_visible(!self.is_visible());
    }

    pub fn clear_results(&self) {
        while let Some(child) = self.results.first_child() {
            self.results.remove(&child);
        }
    }

    pub fn set_status(&self, text: &str) {
        self.status.set_text(text);
    }

    pub fn prompt_text(&self) -> String {
        let buffer = self.prompt.buffer();
        let (start, end) = buffer.bounds();
        buffer.text(&start, &end, false).to_string()
    }

    pub fn set_prompt_text(&self, text: &str) {
        let buffer = self.prompt.buffer();
        buffer.set_text(text);
    }

    pub fn show_reply(
        &self,
        reply: AskReply,
        get_terminal: impl Fn() -> Option<Terminal> + 'static,
    ) {
        self.clear_results();
        if reply.is_empty() {
            self.set_status("Empty reply.");
            return;
        }

        let runnable = reply.runnable_count();
        if runnable == 0 {
            self.set_status("No runnable code blocks — Apply/Run only appears on shell fences.");
        } else {
            self.set_status(&format!("{runnable} runnable block(s)"));
        }

        let get_terminal = Rc::new(get_terminal);
        for segment in reply.segments {
            match segment {
                ReplySegment::Text(text) => {
                    let text = text.trim();
                    if text.is_empty() {
                        continue;
                    }
                    let label = Label::new(Some(text));
                    label.set_wrap(true);
                    label.set_xalign(0.0);
                    label.set_selectable(true);
                    label.add_css_class("ask-text");
                    self.results.append(&label);
                }
                ReplySegment::Code { code, .. } => {
                    let row = GtkBox::new(Orientation::Vertical, 6);
                    row.add_css_class("ask-command-row");

                    let label = Label::new(Some(&code));
                    label.set_wrap(true);
                    label.set_xalign(0.0);
                    label.set_selectable(true);
                    label.add_css_class("ask-command-text");

                    let buttons = GtkBox::new(Orientation::Horizontal, 6);
                    buttons.set_halign(Align::End);

                    let apply = Button::with_label("Apply");
                    apply.add_css_class("ask-apply");
                    apply.set_tooltip_text(Some("Paste into the terminal (no Enter)"));

                    let run = Button::with_label("Run");
                    run.add_css_class("ask-apply");
                    run.add_css_class("ask-run");
                    run.set_tooltip_text(Some("Paste and send Enter"));

                    let code_apply = code.clone();
                    let get_term = get_terminal.clone();
                    apply.connect_clicked(move |_| {
                        if let Some(term) = get_term() {
                            terminal_tab::feed_text(&term, &code_apply);
                            term.grab_focus();
                        }
                    });

                    let code_run = code;
                    let get_term = get_terminal.clone();
                    run.connect_clicked(move |_| {
                        if let Some(term) = get_term() {
                            let mut payload = code_run.clone();
                            if !payload.ends_with('\n') {
                                payload.push('\n');
                            }
                            terminal_tab::feed_text(&term, &payload);
                            term.grab_focus();
                        }
                    });

                    buttons.append(&apply);
                    buttons.append(&run);
                    row.append(&label);
                    row.append(&buttons);
                    self.results.append(&row);
                }
            }
        }
    }

    /// Run an Ask query (from the panel button or an in-shell prefix).
    pub fn run_prompt(
        panel: &Rc<Self>,
        prompt: &str,
        settings: &AppSettings,
        get_terminal: impl Fn() -> Option<Terminal> + 'static,
        open_settings: impl Fn() + 'static,
    ) {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            panel.set_status("Enter a prompt first.");
            return;
        }
        if !settings.llm_configured() {
            panel.set_status("Configure endpoint and API key in Settings.");
            open_settings();
            return;
        }
        if *panel.busy.borrow() {
            panel.set_status("Already waiting on a reply…");
            return;
        }

        panel.set_prompt_text(prompt);
        panel.set_visible(true);
        panel.set_status("Thinking…");
        panel.clear_results();
        panel.ask_btn.set_sensitive(false);
        *panel.busy.borrow_mut() = true;

        let context = get_terminal()
            .map(|term| TerminalContext::from_terminal(&term))
            .unwrap_or_default();

        let (tx, rx) = mpsc::channel::<Result<AskReply, String>>();
        let settings_c = settings.clone();
        let prompt_owned = prompt.to_string();
        thread::spawn(move || {
            let _ = tx.send(llm::ask(&settings_c, &prompt_owned, &context));
        });

        let get_terminal = Rc::new(get_terminal);
        glib::timeout_add_local(
            std::time::Duration::from_millis(50),
            clone!(
                #[strong]
                panel,
                #[strong]
                get_terminal,
                move || {
                    match rx.try_recv() {
                        Ok(Ok(reply)) => {
                            *panel.busy.borrow_mut() = false;
                            panel.ask_btn.set_sensitive(true);
                            let gt = get_terminal.clone();
                            panel.show_reply(reply, move || gt());
                            glib::ControlFlow::Break
                        }
                        Ok(Err(err)) => {
                            *panel.busy.borrow_mut() = false;
                            panel.ask_btn.set_sensitive(true);
                            panel.set_status(&err);
                            glib::ControlFlow::Break
                        }
                        Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            *panel.busy.borrow_mut() = false;
                            panel.ask_btn.set_sensitive(true);
                            panel.set_status("Request cancelled.");
                            glib::ControlFlow::Break
                        }
                    }
                }
            ),
        );
    }

    /// Wire the Ask submit button (must be called once after build).
    pub fn connect_ask(
        panel: &Rc<Self>,
        get_settings: impl Fn() -> AppSettings + 'static,
        get_terminal: impl Fn() -> Option<Terminal> + 'static,
        open_settings: impl Fn() + 'static,
    ) {
        let get_terminal = Rc::new(get_terminal);
        let open_settings = Rc::new(open_settings);

        panel.ask_btn.connect_clicked(clone!(
            #[strong]
            panel,
            #[strong]
            get_terminal,
            #[strong]
            open_settings,
            move |_| {
                let settings = get_settings();
                let prompt = panel.prompt_text();
                let gt = get_terminal.clone();
                let os = open_settings.clone();
                Self::run_prompt(&panel, &prompt, &settings, move || gt(), move || os());
            }
        ));
    }
}

/// Run an Ask query inline in the terminal (no Ask side panel).
/// Caller should already have cleared the typed `?? …` line and stopped Enter.
pub fn run_shell_ask(
    terminal: &Terminal,
    question: &str,
    settings: &AppSettings,
    open_settings: impl Fn() + 'static,
) {
    let question = question.trim();
    if question.is_empty() {
        return;
    }
    if !settings.llm_configured() {
        terminal_tab::feed_output(
            terminal,
            "\r\n\x1b[33mask:\x1b[0m set endpoint + API key in Settings first.\r\n",
        );
        // Redraw shell prompt.
        terminal_tab::feed_text(terminal, "\n");
        open_settings();
        return;
    }

    let context = TerminalContext::from_terminal(terminal);
    terminal_tab::feed_output(
        terminal,
        &format!("\r\n\x1b[1;36m??\x1b[0m {question}\r\n\x1b[2m…\x1b[0m\r\n"),
    );

    let (tx, rx) = mpsc::channel::<Result<AskReply, String>>();
    let settings_c = settings.clone();
    let question_owned = question.to_string();
    thread::spawn(move || {
        let _ = tx.send(llm::ask(&settings_c, &question_owned, &context));
    });

    let term = terminal.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(Ok(reply)) => {
                terminal_tab::feed_output(&term, &format_reply_for_terminal(&reply));
                // Empty Enter so the shell reprints a clean prompt under the answer.
                terminal_tab::feed_text(&term, "\n");
                glib::ControlFlow::Break
            }
            Ok(Err(err)) => {
                terminal_tab::feed_output(
                    &term,
                    &format!("\x1b[31mask error:\x1b[0m {err}\r\n"),
                );
                terminal_tab::feed_text(&term, "\n");
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                terminal_tab::feed_output(&term, "\x1b[31mask:\x1b[0m request cancelled.\r\n");
                terminal_tab::feed_text(&term, "\n");
                glib::ControlFlow::Break
            }
        }
    });
}

fn format_reply_for_terminal(reply: &AskReply) -> String {
    let mut out = String::new();
    if reply.is_empty() {
        out.push_str("\x1b[2m(empty reply)\x1b[0m\r\n");
        return out;
    }

    for segment in &reply.segments {
        match segment {
            ReplySegment::Text(text) => {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                for line in text.lines() {
                    out.push_str(line);
                    out.push_str("\r\n");
                }
            }
            ReplySegment::Code { code, .. } => {
                out.push_str("\x1b[32m");
                for line in code.lines() {
                    out.push_str("  ");
                    out.push_str(line);
                    out.push_str("\r\n");
                }
                out.push_str("\x1b[0m");
            }
        }
    }
    out
}

/// Build Ask toolbar button for the top bar.
pub fn build_ask_button() -> Button {
    let btn = Button::with_label("Ask");
    btn.add_css_class("ask-btn");
    btn.set_tooltip_text(Some("Ask AI for commands"));
    btn.set_focus_on_click(false);
    btn
}

/// Read the current input line from a VTE (prompt + typed text up to the cursor).
pub fn current_input_line(terminal: &Terminal) -> Option<String> {
    let (col, row) = terminal.cursor_position();
    let (text, _) = terminal.text_range_format(Format::Text, row, 0, row, col.max(0));
    text.map(|t| t.to_string())
        .filter(|t| !t.trim().is_empty())
}

/// If `line` is an Ask request using `prefix`, return the question text.
///
/// Works with classic prompts (`$`, `%`) and fancy ones (`➜`, `❯`, oh-my-zsh), by
/// locating the ask prefix as its own token (start-of-line or after whitespace).
///
/// Example: `➜  proj git:(main) ✗ ?? list large files` → `list large files`.
pub fn extract_shell_ask_query(line: &str, prefix: &str) -> Option<String> {
    let prefix = prefix.trim();
    if prefix.is_empty() {
        return None;
    }
    let line = line.trim_end_matches(['\r', '\n']);

    // Prefer last prefix token so prompts that contain the same characters earlier
    // (unlikely) don't steal the match.
    let mut found_at: Option<usize> = None;
    let mut from = 0usize;
    while from <= line.len() {
        let Some(rel) = line[from..].find(prefix) else {
            break;
        };
        let abs = from + rel;
        let before_ok = if abs == 0 {
            true
        } else {
            line[..abs]
                .chars()
                .next_back()
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
        };
        if before_ok {
            found_at = Some(abs);
        }
        from = abs + prefix.len().max(1);
    }

    let start = found_at?;
    let rest = line[start + prefix.len()..].trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

/// Clear the current readline-style input (Ctrl+U) so the Ask line never runs in the shell.
pub fn clear_shell_input_line(terminal: &Terminal) {
    // ASCII NAK — kill-to-start-of-line in emacs/readline (bash/zsh/fish defaults).
    terminal_tab::feed_text(terminal, "\u{15}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_after_bash_prompt() {
        let q = extract_shell_ask_query("md@host:~/proj$ ?? list files", "??").unwrap();
        assert_eq!(q, "list files");
    }

    #[test]
    fn extracts_after_oh_my_zsh_prompt() {
        let q = extract_shell_ask_query(
            "➜  terminal-emulator git:(main) ✗ ?? how do I find large files",
            "??",
        )
        .unwrap();
        assert_eq!(q, "how do I find large files");
    }

    #[test]
    fn extracts_hash_prefix() {
        let q = extract_shell_ask_query("host% #? show disks", "#?").unwrap();
        assert_eq!(q, "show disks");
    }

    #[test]
    fn ignores_non_prefix_commands() {
        assert!(extract_shell_ask_query("md@host:~$ ls -la", "??").is_none());
    }

    #[test]
    fn ignores_empty_question() {
        assert!(extract_shell_ask_query("md@host:~$ ??   ", "??").is_none());
    }

    #[test]
    fn ignores_glob_inside_other_text() {
        // `??` must be its own token, not mid-word.
        assert!(extract_shell_ask_query("echo foo??bar", "??").is_none());
    }
}
