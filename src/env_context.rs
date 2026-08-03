use std::env;
use vte4::prelude::*;
use vte4::{Format, Terminal};

/// High-level guess of what the active terminal pane is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvKind {
    Shell,
    Postgres,
    Mysql,
    Sqlite,
    Mongo,
    Redis,
    Python,
    Node,
    Ruby,
    Docker,
    /// Could not classify beyond "a terminal".
    Unknown,
}

impl Default for EnvKind {
    fn default() -> Self {
        EnvKind::Unknown
    }
}

impl EnvKind {
    pub fn label(self) -> &'static str {
        match self {
            EnvKind::Shell => "interactive shell",
            EnvKind::Postgres => "PostgreSQL (psql)",
            EnvKind::Mysql => "MySQL / MariaDB client",
            EnvKind::Sqlite => "SQLite shell",
            EnvKind::Mongo => "MongoDB shell",
            EnvKind::Redis => "Redis CLI",
            EnvKind::Python => "Python REPL",
            EnvKind::Node => "Node.js REPL",
            EnvKind::Ruby => "Ruby / IRB",
            EnvKind::Docker => "Docker-related session",
            EnvKind::Unknown => "generic Linux terminal",
        }
    }
}

/// Snapshot of the active terminal used to build an Ask environment pre-prompt.
#[derive(Debug, Clone, Default)]
pub struct TerminalContext {
    pub shell: String,
    pub cwd: Option<String>,
    pub title: Option<String>,
    pub recent_output: Option<String>,
    pub kind: EnvKind,
}

impl TerminalContext {
    /// Collect environment hints from the focused VTE (must run on the GTK thread).
    pub fn from_terminal(terminal: &Terminal) -> Self {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let title = terminal
            .window_title()
            .map(|t| t.to_string())
            .filter(|t| !t.is_empty());
        let cwd = terminal
            .current_directory_uri()
            .and_then(|uri| file_uri_to_path(&uri));
        let recent_output = recent_screen_text(terminal, 40);

        let mut ctx = Self {
            shell,
            cwd,
            title,
            recent_output,
            kind: EnvKind::Unknown,
        };
        ctx.kind = detect_kind(&ctx);
        ctx
    }

    /// Build a context pre-prompt. Always includes at least a generic terminal baseline.
    pub fn build_pre_prompt(&self) -> String {
        let shell_name = shell_basename(&self.shell);
        let mut lines = Vec::new();

        lines.push("## Active terminal environment".to_string());
        lines.push(format!("- Host shell: {shell_name} ({})", self.shell));
        if let Some(cwd) = &self.cwd {
            lines.push(format!("- Working directory: {cwd}"));
        } else {
            lines.push("- Working directory: unknown".into());
        }
        if let Some(title) = &self.title {
            lines.push(format!("- Window / tab title: {title}"));
        }

        match self.kind {
            EnvKind::Unknown => {
                lines.push(
                    "- Detected session: could not identify a specific REPL or DB client."
                        .into(),
                );
                lines.push(
                    "- Assume a general Linux terminal. Prefer portable shell commands for this shell."
                        .into(),
                );
            }
            other => {
                lines.push(format!("- Detected session: {}", other.label()));
                lines.push(format!(
                    "- Tailor answers for {}. If this is a DB/REPL prompt, prefer that language's statements over outer-shell wrappers unless the user asks for shell.",
                    other.label()
                ));
            }
        }

        if let Some(out) = &self.recent_output {
            let clipped = clip_output(out, 1200);
            if !clipped.trim().is_empty() {
                lines.push("- Recent visible terminal text (may include prompts/output):".into());
                lines.push("```".into());
                lines.push(clipped);
                lines.push("```".into());
            }
        }

        lines.push(
            "- Treat this block as a mandatory pre-prompt for the user's question."
                .into(),
        );

        lines.join("\n")
    }
}

fn shell_basename(shell: &str) -> &str {
    std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell)
}

fn file_uri_to_path(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://")?;
    let decoded = percent_decode(path);
    if decoded.is_empty() {
        None
    } else {
        Some(decoded)
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn recent_screen_text(terminal: &Terminal, max_rows: i64) -> Option<String> {
    let (col, row) = terminal.cursor_position();
    let start_row = (row - max_rows + 1).max(0);
    let (text, _) = terminal.text_range_format(Format::Text, start_row, 0, row, col.max(0));
    text.map(|t| t.to_string()).filter(|t| !t.trim().is_empty())
}

fn clip_output(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim_end();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let start: String = trimmed
        .chars()
        .rev()
        .take(max_chars)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("…{start}")
}

/// Detect session kind from title + recent output (+ shell name as weak signal).
pub fn detect_kind(ctx: &TerminalContext) -> EnvKind {
    let haystack = {
        let mut s = String::new();
        if let Some(t) = &ctx.title {
            s.push_str(t);
            s.push('\n');
        }
        if let Some(o) = &ctx.recent_output {
            s.push_str(o);
        }
        s.to_ascii_lowercase()
    };

    if haystack.is_empty() {
        return if !ctx.shell.is_empty() {
            EnvKind::Shell
        } else {
            EnvKind::Unknown
        };
    }

    if contains_any(
        &haystack,
        &["psql", "postgres=#", "postgres=>", "postgres-#"],
    ) {
        return EnvKind::Postgres;
    }
    if contains_any(&haystack, &["mysql>", "mariadb>", "mysql ", "mariadb "]) {
        return EnvKind::Mysql;
    }
    if contains_any(&haystack, &["sqlite>", "sqlite3"]) {
        return EnvKind::Sqlite;
    }
    if contains_any(&haystack, &["mongosh", "mongo>", "mongodb"]) {
        return EnvKind::Mongo;
    }
    if contains_any(&haystack, &["redis-cli", "redis>"]) {
        return EnvKind::Redis;
    }
    if (haystack.contains(">>>") || haystack.contains("ipython"))
        && !haystack.contains("node>")
    {
        return EnvKind::Python;
    }
    if contains_any(&haystack, &["node>", "nodejs"]) {
        return EnvKind::Node;
    }
    if contains_any(&haystack, &["irb(", "irb>"]) {
        return EnvKind::Ruby;
    }
    if contains_any(&haystack, &["docker", "container"]) {
        return EnvKind::Docker;
    }

    if !ctx.shell.is_empty() {
        return EnvKind::Shell;
    }

    EnvKind::Unknown
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_psql_from_title() {
        let ctx = TerminalContext {
            shell: "/bin/zsh".into(),
            title: Some("psql mydb".into()),
            ..Default::default()
        };
        assert_eq!(detect_kind(&ctx), EnvKind::Postgres);
    }

    #[test]
    fn detects_mysql_from_output() {
        let ctx = TerminalContext {
            shell: "/bin/bash".into(),
            recent_output: Some("Welcome\nmysql> SELECT 1;\n".into()),
            ..Default::default()
        };
        assert_eq!(detect_kind(&ctx), EnvKind::Mysql);
    }

    #[test]
    fn empty_hints_still_shell_when_shell_known() {
        let ctx = TerminalContext {
            shell: "/usr/bin/zsh".into(),
            ..Default::default()
        };
        assert_eq!(detect_kind(&ctx), EnvKind::Shell);
    }

    #[test]
    fn pre_prompt_always_mentions_terminal() {
        let ctx = TerminalContext {
            shell: "/bin/bash".into(),
            kind: EnvKind::Unknown,
            ..Default::default()
        };
        let prompt = ctx.build_pre_prompt();
        assert!(prompt.contains("Active terminal environment"));
        assert!(prompt.contains("general Linux terminal") || prompt.contains("bash"));
    }

    #[test]
    fn pre_prompt_mentions_postgres() {
        let ctx = TerminalContext {
            shell: "/bin/bash".into(),
            kind: EnvKind::Postgres,
            cwd: Some("/home/me/proj".into()),
            title: Some("psql".into()),
            ..Default::default()
        };
        let prompt = ctx.build_pre_prompt();
        assert!(prompt.contains("PostgreSQL"));
        assert!(prompt.contains("/home/me/proj"));
    }
}
