use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, GestureClick, UriLauncher};
use std::cell::RefCell;
use std::collections::HashMap;
use vte4::prelude::*;
use vte4::{Regex, Terminal};

/// PCRE2 compile flag required by VTE match regexes.
const PCRE2_MULTILINE: u32 = 0x400;

const DEFAULT_LINK_COLOR: &str = "#6cb6ff";

thread_local! {
    static LINK_COLORS: RefCell<HashMap<usize, String>> = RefCell::new(HashMap::new());
}

/// Detect http(s), www., and file:// URLs for Ctrl+click and middle-click open.
const URL_MATCH_PATTERN: &str = r"(?:(?:https?|ftp)://|www\.|file://)[-A-Za-z0-9+&@#/%?=~_|!:,.;]+[-A-Za-z0-9+&@#/%=~_|]";

pub fn set_link_color(terminal: &Terminal, color: &str) {
    LINK_COLORS.with(|colors| {
        colors
            .borrow_mut()
            .insert(terminal.as_ptr() as usize, color.to_string());
    });
}

pub fn link_color_for(terminal: &Terminal) -> String {
    LINK_COLORS.with(|colors| {
        colors
            .borrow()
            .get(&(terminal.as_ptr() as usize))
            .cloned()
            .unwrap_or_else(|| DEFAULT_LINK_COLOR.to_string())
    })
}

pub fn setup(terminal: &Terminal, link_color: &str) {
    set_link_color(terminal, link_color);
    terminal.set_allow_hyperlink(true);

    if let Ok(regex) = Regex::for_match(URL_MATCH_PATTERN, PCRE2_MULTILINE) {
        let tag = terminal.match_add_regex(&regex, 0);
        terminal.match_set_cursor_name(tag, "pointer");
    }
}

pub fn wire_clicks(terminal: &Terminal, window: &ApplicationWindow) {
    let click = GestureClick::new();
    click.set_button(0); // all buttons
    click.connect_pressed(glib::clone!(
        #[strong]
        terminal,
        #[strong]
        window,
        move |gesture, _n_press, x, y| {
            let button = gesture.current_button();
            let ctrl = gesture
                .current_event()
                .map(|event| event.modifier_state().contains(gdk::ModifierType::CONTROL_MASK))
                .unwrap_or(false);

            // Left-click opens only with Ctrl (preserves drag-to-select). Middle-click always opens.
            if button == gdk::BUTTON_PRIMARY as u32 && !ctrl {
                return;
            }
            if button != gdk::BUTTON_PRIMARY as u32 && button != gdk::BUTTON_MIDDLE as u32 {
                return;
            }

            if let Some(uri) = terminal.check_hyperlink_at(x, y) {
                open_uri(uri.as_str(), &window);
                return;
            }

            if let (Some(text), _) = terminal.check_match_at(x, y) {
                open_uri(&normalize_uri(text.as_str()), &window);
            }
        }
    ));
    terminal.add_controller(click);
}

pub fn linkify(text: &str, link_color: &str) -> String {
    let (r, g, b) = parse_hex_rgb(link_color);
    let style_off = "\x1b[0m\x1b]8;;\x1b\\";

    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'\x1b' {
            if let Some(end) = find_escape_end(bytes, i) {
                out.push_str(&text[i..=end]);
                i = end + 1;
                continue;
            }
        }

        if let Some((url, end)) = scan_url(text, i) {
            let uri = normalize_uri(&url);
            out.push_str(&format!(
                "\x1b]8;;{uri}\x1b\\\x1b[4m\x1b[38;2;{r};{g};{b}m{url}{style_off}",
                uri = uri,
                r = r,
                g = g,
                b = b,
                url = url,
                style_off = style_off,
            ));
            i = end;
        } else {
            out.push(text[i..].chars().next().unwrap_or('\0'));
            i += text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        }
    }

    out
}

fn find_escape_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'\x1b') {
        return None;
    }
    if bytes.get(start + 1) == Some(&b']') {
        // OSC sequence ends at BEL (\x07) or ST (\x1b\\).
        for j in (start + 2)..bytes.len() {
            if bytes[j] == 0x07 {
                return Some(j);
            }
            if bytes[j] == b'\x1b' && bytes.get(j + 1) == Some(&b'\\') {
                return Some(j + 1);
            }
        }
        return None;
    }
    // CSI / single-char escapes.
    let mut j = start + 1;
    while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
        j += 1;
    }
    if j > start + 1 {
        Some(j - 1)
    } else {
        None
    }
}

fn scan_url(text: &str, start: usize) -> Option<(String, usize)> {
    let rest = &text[start..];
    let (skip,) = if rest.starts_with("http://") {
        (7,)
    } else if rest.starts_with("https://") {
        (8,)
    } else if rest.starts_with("ftp://") {
        (6,)
    } else if rest.starts_with("file://") {
        (7,)
    } else if rest.starts_with("www.") {
        (4,)
    } else {
        return None;
    };

    if start > 0 {
        let prev = text[..start].chars().last().unwrap_or(' ');
        if !prev.is_whitespace() && !matches!(prev, '"' | '\'' | '(' | '[' | '<' | '{' | '|') {
            return None;
        }
    }

    let mut end = start + skip;
    while end < text.len() {
        let ch = text[end..].chars().next()?;
        if is_url_char(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }

    while end > start + skip {
        let ch = text[..end].chars().last().unwrap();
        if matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '>' | '"' | '\'') {
            end -= ch.len_utf8();
        } else {
            break;
        }
    }

    if end <= start + skip {
        return None;
    }

    Some((text[start..end].to_string(), end))
}

fn is_url_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '-' | '+' | '&' | '@' | '#' | '/' | '%' | '?' | '=' | '~' | '_' | '.' | ',' | ';' | ':'
                | '!' | '|'
        )
}

fn parse_hex_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim().trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(108);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(182);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(6);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(6);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(6);
            (r * 17, g * 17, b * 17)
        }
        _ => (108, 182, 255),
    }
}

fn normalize_uri(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with("www.") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    }
}

fn open_uri(uri: &str, window: &ApplicationWindow) {
    let uri = uri.trim();
    if uri.is_empty() {
        return;
    }
    let uri_owned = uri.to_string();
    let launcher = UriLauncher::new(uri);
    launcher.launch(
        Some(window),
        None::<&gio::Cancellable>,
        move |result| {
            if let Err(err) = result {
                eprintln!("failed to open link {uri_owned}: {err}");
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linkify_wraps_http_url_with_color_and_hyperlink() {
        let out = linkify("see https://example.com/path ok", "#6cb6ff");
        assert!(out.contains("\x1b]8;;https://example.com/path\x1b\\"));
        assert!(out.contains("\x1b[4m"));
        assert!(out.contains("\x1b[38;2;108;182;255m"));
        assert!(out.contains("https://example.com/path"));
    }

    #[test]
    fn linkify_skips_existing_escape_sequences() {
        let out = linkify("\x1b[31mhttps://example.com\x1b[0m", "#6cb6ff");
        assert_eq!(out, "\x1b[31mhttps://example.com\x1b[0m");
    }

    #[test]
    fn normalize_uri_adds_https_for_www() {
        assert_eq!(
            normalize_uri("www.example.com"),
            "https://www.example.com"
        );
    }
}
