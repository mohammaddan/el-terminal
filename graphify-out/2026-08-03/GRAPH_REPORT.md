# Graph Report - terminal-emulator  (2026-08-03)

## Corpus Check
- 14 files · ~19,658 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 249 nodes · 525 edges · 13 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `bb46e999`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_terminal_tab.rs|terminal_tab.rs]]
- [[_COMMUNITY_tab_bar.rs|tab_bar.rs]]
- [[_COMMUNITY_attach_context_menu|attach_context_menu]]
- [[_COMMUNITY_Rc|Rc]]
- [[_COMMUNITY_Terminal Emulator|Terminal Emulator]]
- [[_COMMUNITY_Tab|Tab]]
- [[_COMMUNITY_build_ui|build_ui]]
- [[_COMMUNITY_blur.rs|blur.rs]]
- [[_COMMUNITY_app.rs|app.rs]]
- [[_COMMUNITY_hit_interactive_child|hit_interactive_child]]
- [[_COMMUNITY_env_context.rs|env_context.rs]]

## God Nodes (most connected - your core abstractions)
1. `AppState` - 32 edges
2. `AppSettings` - 21 edges
3. `AskPanel` - 19 edges
4. `parse_reply()` - 13 edges
5. `TerminalContext` - 10 edges
6. `ask()` - 10 edges
7. `parse_commands()` - 10 edges
8. `extract_assistant_content()` - 10 edges
9. `Tab` - 9 edges
10. `build_ui()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `AppState` --references--> `AskPanel`  [EXTRACTED]
  src/app.rs → src/ask.rs
- `AppState` --references--> `AppSettings`  [EXTRACTED]
  src/app.rs → src/settings.rs
- `ask()` --references--> `TerminalContext`  [EXTRACTED]
  src/llm.rs → src/env_context.rs
- `ask()` --references--> `AppSettings`  [EXTRACTED]
  src/llm.rs → src/settings.rs
- `apply_font()` --references--> `AppSettings`  [EXTRACTED]
  src/terminal_tab.rs → src/settings.rs

## Import Cycles
- None detected.

## Communities (13 total, 0 thin omitted)

### Community 0 - "terminal_tab.rs"
Cohesion: 0.20
Nodes (18): Path, RGBA, RgbaF, apply_font(), apply_palette(), copy_selection(), create_terminal(), default_title() (+10 more)

### Community 1 - "tab_bar.rs"
Cohesion: 0.36
Nodes (8): build_menu_button(), build_new_tab_button(), build_status_dot(), build_tab_pill(), build_window_controls(), Button, GtkBox, Label

### Community 2 - "attach_context_menu"
Cohesion: 0.33
Nodes (6): attach_context_menu(), install_window_actions(), ApplicationWindow, Fn, IsA, Widget

### Community 3 - "Rc"
Cohesion: 0.11
Nodes (19): AskPanel, build_ask_button(), clear_shell_input_line(), current_input_line(), extract_shell_ask_query(), extracts_after_bash_prompt(), extracts_after_oh_my_zsh_prompt(), extracts_hash_prefix() (+11 more)

### Community 4 - "Terminal Emulator"
Cohesion: 0.33
Nodes (5): Build & run, Dependencies, Features, Layout, Terminal Emulator

### Community 5 - "Tab"
Cohesion: 0.11
Nodes (45): Application, Cell, Orientation, Paned, active_terminal(), add_tab(), adjust_font_size(), apply_settings_to_all() (+37 more)

### Community 6 - "build_ui"
Cohesion: 0.16
Nodes (33): ApiError, ask(), AskReply, assemble_sse_content(), ChatChoice, ChatMessage, ChatReply, ChatRequest (+25 more)

### Community 7 - "blur.rs"
Cohesion: 0.14
Nodes (24): Connection, Dispatch, Event, EventQueue, ExtBackgroundEffectManagerV1, ExtBackgroundEffectSurfaceV1, GlobalListContents, OrgKdeKwinBlur (+16 more)

### Community 8 - "app.rs"
Cohesion: 0.31
Nodes (9): Context, DrawingArea, ImageSurface, build_chrome_background(), build_noise_surface(), hash2(), NoiseCache, rounded_rect() (+1 more)

### Community 9 - "hit_interactive_child"
Cohesion: 0.13
Nodes (15): PathBuf, AppSettings, default_ask_prefix(), Default, Result, Self, String, labeled_row() (+7 more)

### Community 12 - "env_context.rs"
Cohesion: 0.17
Nodes (20): clip_output(), contains_any(), detect_kind(), detects_mysql_from_output(), detects_psql_from_title(), empty_hints_still_shell_when_shell_known(), EnvKind, file_uri_to_path() (+12 more)

## Knowledge Gaps
- **4 isolated node(s):** `Features`, `Dependencies`, `Build & run`, `Layout`
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppSettings` connect `hit_interactive_child` to `terminal_tab.rs`, `Rc`, `Tab`, `build_ui`?**
  _High betweenness centrality (0.191) - this node is a cross-community bridge._
- **Why does `TerminalContext` connect `env_context.rs` to `Rc`, `build_ui`?**
  _High betweenness centrality (0.149) - this node is a cross-community bridge._
- **Why does `AppState` connect `Tab` to `hit_interactive_child`, `Rc`?**
  _High betweenness centrality (0.109) - this node is a cross-community bridge._
- **What connects `Features`, `Dependencies`, `Build & run` to the rest of the system?**
  _4 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Rc` be split into smaller, more focused modules?**
  _Cohesion score 0.11092436974789915 - nodes in this community are weakly interconnected._
- **Should `Tab` be split into smaller, more focused modules?**
  _Cohesion score 0.11304347826086956 - nodes in this community are weakly interconnected._
- **Should `blur.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13538461538461538 - nodes in this community are weakly interconnected._