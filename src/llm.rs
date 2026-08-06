use crate::env_context::TerminalContext;
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SYSTEM_PROMPT: &str = r#"You are a shell / terminal assistant for a Linux terminal emulator.
An "Active terminal environment" section is appended to this system prompt. Always respect it.

Answer briefly in plain text when helpful.
Put every runnable command or statement inside a fenced code block so the user can Apply/Run it, for example:

```bash
ls -la
```

Rules:
- Prefer commands that fit the detected session (SQL in psql/mysql, Python in a REPL, shell otherwise).
- If the environment could not be identified, assume a general Linux terminal and use portable shell commands for the listed shell.
- Prefer safe, non-destructive commands unless the user clearly asks otherwise.
- Do not put explanations inside code fences.
- You may also reply with JSON {"commands":["..."]} instead of prose; each entry is a runnable command.
- Always follow the Active terminal environment section when present."#;

/// A piece of an Ask reply. Only shell [`ReplySegment::Code`] blocks get Apply/Run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplySegment {
    /// Prose or non-shell content — display only, no Apply/Run.
    Text(String),
    /// Shell / terminal code the user can Apply/Run into the terminal.
    Code { language: String, code: String },
}

impl ReplySegment {
    pub fn is_runnable(&self) -> bool {
        matches!(self, ReplySegment::Code { .. })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AskReply {
    pub segments: Vec<ReplySegment>,
}

impl AskReply {
    pub fn runnable_count(&self) -> usize {
        self.segments.iter().filter(|s| s.is_runnable()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
            || self.segments.iter().all(|s| match s {
                ReplySegment::Text(t) => t.trim().is_empty(),
                ReplySegment::Code { code, .. } => code.trim().is_empty(),
            })
    }
}

fn is_shell_language(lang: &str) -> bool {
    matches!(
        lang,
        "" | "bash"
            | "sh"
            | "shell"
            | "zsh"
            | "fish"
            | "console"
            | "terminal"
            | "powershell"
            | "pwsh"
            | "cmd"
    )
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: Option<ChatReply>,
    text: Option<String>,
    delta: Option<StreamDelta>,
}

#[derive(Deserialize)]
struct ChatReply {
    content: Option<String>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Deserialize)]
struct ApiError {
    message: Option<String>,
}

#[derive(Deserialize)]
struct CommandsPayload {
    commands: Vec<String>,
}

/// Call an OpenAI-compatible chat completions endpoint.
/// Accepts both streaming (SSE) and non-streaming JSON responses.
/// When `settings.ask_share_terminal_context` is true, `context` is appended
/// as an environment pre-prompt; otherwise only the generic system prompt is sent.
pub fn ask(
    settings: &AppSettings,
    prompt: &str,
    context: &TerminalContext,
) -> Result<AskReply, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("Prompt is empty".into());
    }
    if settings.llm_api_key.trim().is_empty() {
        return Err("Set an API key in Settings".into());
    }

    let base = settings.llm_endpoint.trim_end_matches('/');
    let url = format!("{base}/chat/completions");

    let system = if settings.ask_share_terminal_context {
        format!("{SYSTEM_PROMPT}\n\n{}", context.build_pre_prompt())
    } else {
        format!(
            "{SYSTEM_PROMPT}\n\n## Active terminal environment\n\
             - No live terminal context was shared for this request.\n\
             - Assume a general Linux terminal and portable shell commands."
        )
    };

    let body = ChatRequest {
        model: &settings.llm_model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: &system,
            },
            ChatMessage {
                role: "user",
                content: prompt,
            },
        ],
        temperature: 0.2,
        stream: true,
    };

    let json_body = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let response = ureq::post(&url)
        .header(
            "Authorization",
            &format!("Bearer {}", settings.llm_api_key.trim()),
        )
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream, application/json")
        .send(&json_body)
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let text = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format_api_error(status.as_u16(), &text));
    }

    let content = extract_assistant_content(&text, &content_type)?;
    parse_reply(&content)
}

/// Runnable command strings only (tests / helpers).
pub fn parse_commands(content: &str) -> Result<Vec<String>, String> {
    let reply = parse_reply(content)?;
    let cmds: Vec<String> = reply
        .segments
        .into_iter()
        .filter_map(|s| match s {
            ReplySegment::Code { code, .. } => Some(code),
            ReplySegment::Text(_) => None,
        })
        .collect();
    if cmds.is_empty() {
        Err("No runnable commands in reply".into())
    } else {
        Ok(cmds)
    }
}

fn format_api_error(status: u16, text: &str) -> String {
    if let Ok(err) = serde_json::from_str::<ChatResponse>(text) {
        if let Some(api_err) = err.error {
            if let Some(msg) = api_err.message {
                return format!("API error ({status}): {msg}");
            }
        }
    }
    format!("API error ({status}): {text}")
}

fn extract_assistant_content(body: &str, content_type: &str) -> Result<String, String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err("Empty API response".into());
    }

    let looks_like_sse = content_type.contains("text/event-stream")
        || trimmed.lines().any(|l| {
            let l = l.trim();
            l.starts_with("data:") || l == "data: [DONE]"
        });

    if looks_like_sse {
        return assemble_sse_content(trimmed);
    }

    extract_json_message_content(trimmed)
}

fn extract_json_message_content(text: &str) -> Result<String, String> {
    let parsed: ChatResponse =
        serde_json::from_str(text).map_err(|e| format!("Invalid API JSON: {e}"))?;

    if let Some(api_err) = parsed.error {
        return Err(api_err
            .message
            .unwrap_or_else(|| "Unknown API error".into()));
    }

    let content = parsed
        .choices
        .and_then(|c| c.into_iter().next())
        .map(|c| {
            c.message
                .and_then(|m| m.content)
                .or(c.text)
                .or_else(|| c.delta.and_then(|d| d.content))
                .unwrap_or_default()
        })
        .unwrap_or_default();

    Ok(content)
}

fn assemble_sse_content(body: &str) -> Result<String, String> {
    let mut content = String::new();
    let mut saw_chunk = false;
    let mut last_error: Option<String> = None;

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        let Some(payload) = line.strip_prefix("data:") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }

        if let Ok(full) = serde_json::from_str::<ChatResponse>(payload) {
            if let Some(api_err) = full.error {
                last_error = Some(
                    api_err
                        .message
                        .unwrap_or_else(|| "Unknown API error".into()),
                );
                continue;
            }
            if let Some(choices) = full.choices {
                for choice in choices {
                    if let Some(msg) = choice.message.and_then(|m| m.content) {
                        content.push_str(&msg);
                        saw_chunk = true;
                    } else if let Some(text) = choice.text {
                        content.push_str(&text);
                        saw_chunk = true;
                    } else if let Some(delta) = choice.delta.and_then(|d| d.content) {
                        content.push_str(&delta);
                        saw_chunk = true;
                    }
                }
            }
            continue;
        }

        let value: Value = match serde_json::from_str(payload) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(err) = value.get("error") {
            last_error = Some(
                err.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error")
                    .to_string(),
            );
            continue;
        }

        let Some(choices) = value.get("choices").and_then(|c| c.as_array()) else {
            continue;
        };

        for choice in choices {
            if let Some(piece) = choice
                .pointer("/delta/content")
                .and_then(|c| c.as_str())
                .or_else(|| choice.pointer("/message/content").and_then(|c| c.as_str()))
                .or_else(|| choice.get("text").and_then(|c| c.as_str()))
            {
                content.push_str(piece);
                saw_chunk = true;
            }
        }
    }

    if !saw_chunk {
        if let Ok(fallback) = extract_json_message_content(body.trim()) {
            if !fallback.is_empty() {
                return Ok(fallback);
            }
        }
        if let Some(err) = last_error {
            return Err(err);
        }
        return Err("Stream ended with no content".into());
    }

    Ok(content)
}

/// Parse assistant markdown/JSON into text + runnable shell code segments.
pub fn parse_reply(content: &str) -> Result<AskReply, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("Model returned an empty reply".into());
    }

    // Explicit JSON command list → each entry is a runnable code block.
    if let Ok(cmds) = try_parse_json_commands(trimmed) {
        return Ok(AskReply {
            segments: cmds
                .into_iter()
                .map(|code| ReplySegment::Code {
                    language: "bash".into(),
                    code,
                })
                .collect(),
        });
    }

    if let Some(inner) = strip_fence_if_json(trimmed) {
        if let Ok(cmds) = try_parse_json_commands(inner) {
            return Ok(AskReply {
                segments: cmds
                    .into_iter()
                    .map(|code| ReplySegment::Code {
                        language: "bash".into(),
                        code,
                    })
                    .collect(),
            });
        }
    }

    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                let slice = &trimmed[start..=end];
                if let Ok(cmds) = try_parse_json_commands(slice) {
                    let mut segments = Vec::new();
                    let before = trimmed[..start].trim();
                    if !before.is_empty() {
                        segments.push(ReplySegment::Text(before.to_string()));
                    }
                    for code in cmds {
                        segments.push(ReplySegment::Code {
                            language: "bash".into(),
                            code,
                        });
                    }
                    let after = trimmed[end + 1..].trim();
                    if !after.is_empty() {
                        segments.push(ReplySegment::Text(after.to_string()));
                    }
                    return Ok(AskReply { segments });
                }
            }
        }
    }

    Ok(AskReply {
        segments: split_markdown_segments(trimmed),
    })
}

fn try_parse_json_commands(raw: &str) -> Result<Vec<String>, String> {
    if let Ok(payload) = serde_json::from_str::<CommandsPayload>(raw) {
        let cmds: Vec<String> = payload
            .commands
            .into_iter()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if cmds.is_empty() {
            return Err("No commands in JSON".into());
        }
        return Ok(cmds);
    }

    let value: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let Some(commands) = value.get("commands") else {
        return Err("Missing commands field".into());
    };

    let mut out = Vec::new();
    match commands {
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(s) => {
                        let s = s.trim();
                        if !s.is_empty() {
                            out.push(s.to_string());
                        }
                    }
                    Value::Object(obj) => {
                        if let Some(Value::String(s)) = obj.get("command").or_else(|| obj.get("cmd"))
                        {
                            let s = s.trim();
                            if !s.is_empty() {
                                out.push(s.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Value::String(s) => {
            for line in s.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    out.push(line.to_string());
                }
            }
        }
        _ => {}
    }

    if out.is_empty() {
        Err("No commands in JSON".into())
    } else {
        Ok(out)
    }
}

fn strip_fence_if_json(text: &str) -> Option<&str> {
    let text = text.trim();
    if !text.starts_with("```") {
        return None;
    }
    let after = text.find('\n')? + 1;
    let end = text.rfind("```")?;
    if end <= after {
        None
    } else {
        Some(text[after..end].trim())
    }
}

/// Split markdown into text and fenced blocks. Only shell-language fences are runnable.
fn split_markdown_segments(text: &str) -> Vec<ReplySegment> {
    let mut segments = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("```") {
        let before = rest[..start].trim();
        if !before.is_empty() {
            segments.push(ReplySegment::Text(before.to_string()));
        }

        rest = &rest[start + 3..];
        let (lang, body_start) = if let Some(nl) = rest.find('\n') {
            let lang = rest[..nl].trim().to_ascii_lowercase();
            (lang, nl + 1)
        } else {
            (String::new(), 0)
        };
        rest = &rest[body_start..];

        let Some(end) = rest.find("```") else {
            let code = rest.trim();
            if !code.is_empty() {
                push_code_or_text(&mut segments, &lang, code);
            }
            rest = "";
            break;
        };

        let code = rest[..end].trim();
        rest = &rest[end + 3..];
        if !code.is_empty() {
            push_code_or_text(&mut segments, &lang, code);
        }
    }

    let trailing = rest.trim();
    if !trailing.is_empty() {
        segments.push(ReplySegment::Text(trailing.to_string()));
    }

    segments
}

fn push_code_or_text(segments: &mut Vec<ReplySegment>, lang: &str, code: &str) {
    if is_shell_language(lang) {
        segments.push(ReplySegment::Code {
            language: if lang.is_empty() {
                "bash".into()
            } else {
                lang.to_string()
            },
            code: code.to_string(),
        });
    } else {
        // Non-shell fence: show as text, no Apply/Run.
        let labeled = if lang.is_empty() {
            code.to_string()
        } else {
            format!("```{lang}\n{code}\n```")
        };
        segments.push(ReplySegment::Text(labeled));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_commands() {
        let cmds = parse_commands(r#"{"commands":["ls -la","pwd"]}"#).unwrap();
        assert_eq!(cmds, vec!["ls -la", "pwd"]);
    }

    #[test]
    fn parses_fenced_bash_as_runnable() {
        let reply = parse_reply("Here:\n```bash\nls\npwd\n```").unwrap();
        assert_eq!(reply.runnable_count(), 1);
        assert!(matches!(
            &reply.segments[0],
            ReplySegment::Text(t) if t == "Here:"
        ));
        assert!(matches!(
            &reply.segments[1],
            ReplySegment::Code { code, .. } if code == "ls\npwd"
        ));
    }

    #[test]
    fn non_shell_fence_has_no_apply() {
        let reply = parse_reply("Example:\n```python\nprint(1)\n```").unwrap();
        assert_eq!(reply.runnable_count(), 0);
        assert!(reply.segments.iter().all(|s| !s.is_runnable()));
    }

    #[test]
    fn plain_text_has_no_apply() {
        let reply = parse_reply("Just an explanation with no code.").unwrap();
        assert_eq!(reply.runnable_count(), 0);
        assert!(matches!(reply.segments[0], ReplySegment::Text(_)));
    }

    #[test]
    fn extracts_non_stream_json() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"commands\":[\"echo hi\"]}"}}]}"#;
        let content = extract_assistant_content(body, "application/json").unwrap();
        let cmds = parse_commands(&content).unwrap();
        assert_eq!(cmds, vec!["echo hi"]);
    }

    #[test]
    fn extracts_sse_stream() {
        let body = "\
data: {\"choices\":[{\"delta\":{\"content\":\"{\\\"commands\\\"\"}}]}\n\n\
data: {\"choices\":[{\"delta\":{\"content\":\":[\\\"uname -a\\\"]}\"}}]}\n\n\
data: [DONE]\n";
        let content = extract_assistant_content(body, "text/event-stream").unwrap();
        let cmds = parse_commands(&content).unwrap();
        assert_eq!(cmds, vec!["uname -a"]);
    }

    #[test]
    fn extracts_sse_without_content_type_header() {
        let body = "\
data: {\"choices\":[{\"delta\":{\"content\":\"{\\\"commands\\\":[\\\"pwd\\\"]}\"}}]}\n\n\
data: [DONE]\n";
        let content = extract_assistant_content(body, "").unwrap();
        let cmds = parse_commands(&content).unwrap();
        assert_eq!(cmds, vec!["pwd"]);
    }
}
