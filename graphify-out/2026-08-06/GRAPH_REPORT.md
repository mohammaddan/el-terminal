# Graph Report - terminal-emulator  (2026-08-06)

## Corpus Check
- 21 files · ~81,898 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 305 nodes · 621 edges · 20 communities (16 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7da655a3`
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
- [[_COMMUNITY_main.rs|main.rs]]
- [[_COMMUNITY_env_context.rs|env_context.rs]]
- [[_COMMUNITY_postinst|postinst]]
- [[_COMMUNITY_LaunchOptions|LaunchOptions]]
- [[_COMMUNITY_ElTerminalExtension|ElTerminalExtension]]
- [[_COMMUNITY_test-desktop-integration.sh|test-desktop-integration.sh]]
- [[_COMMUNITY_postinst|postinst]]
- [[_COMMUNITY_postrm|postrm]]
- [[_COMMUNITY_prerm|prerm]]

## God Nodes (most connected - your core abstractions)
1. `AppState` - 33 edges
2. `AppSettings` - 23 edges
3. `AskPanel` - 20 edges
4. `parse_reply()` - 13 edges
5. `build_ui()` - 10 edges
6. `TerminalContext` - 10 edges
7. `ask()` - 10 edges
8. `parse_commands()` - 10 edges
9. `extract_assistant_content()` - 10 edges
10. `Tab` - 9 edges

## Surprising Connections (you probably didn't know these)
- `run_shell_ask()` --calls--> `open_settings()`  [INFERRED]
  src/ask.rs → src/app.rs
- `AppState` --references--> `AskPanel`  [EXTRACTED]
  src/app.rs → src/ask.rs
- `AppState` --references--> `AppSettings`  [EXTRACTED]
  src/app.rs → src/settings.rs
- `run_shell_ask()` --references--> `AppSettings`  [EXTRACTED]
  src/ask.rs → src/settings.rs
- `format_reply_for_terminal()` --references--> `AskReply`  [EXTRACTED]
  src/ask.rs → src/llm.rs

## Import Cycles
- None detected.

## Communities (20 total, 4 thin omitted)

### Community 0 - "terminal_tab.rs"
Cohesion: 0.10
Nodes (29): Path, RGBA, RgbaF, AppSettings, default_ask_prefix(), Default, PathBuf, Result (+21 more)

### Community 1 - "tab_bar.rs"
Cohesion: 0.36
Nodes (8): build_menu_button(), build_new_tab_button(), build_status_dot(), build_tab_pill(), build_window_controls(), Button, GtkBox, Label

### Community 2 - "attach_context_menu"
Cohesion: 0.33
Nodes (6): attach_context_menu(), install_window_actions(), ApplicationWindow, Fn, IsA, Widget

### Community 3 - "Rc"
Cohesion: 0.11
Nodes (21): AskPanel, build_ask_button(), clear_shell_input_line(), current_input_line(), extract_shell_ask_query(), extracts_after_bash_prompt(), extracts_after_oh_my_zsh_prompt(), extracts_hash_prefix() (+13 more)

### Community 4 - "Terminal Emulator"
Cohesion: 0.14
Nodes (13): Ask AI setup, Build & run, CLI, Default terminal, Dependencies, Desktop integration (Ubuntu 24.04), El-Terminal, Features (+5 more)

### Community 5 - "Tab"
Cohesion: 0.11
Nodes (46): Application, Cell, Orientation, Paned, active_terminal(), add_tab(), adjust_font_size(), apply_settings_to_all() (+38 more)

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
Cohesion: 0.28
Nodes (8): labeled_row(), open_settings_dialog(), Fn, GtkBox, IsA, Label, section_label(), Window

### Community 11 - "main.rs"
Cohesion: 0.43
Nodes (6): lookup_filename_option(), main(), Option, String, trim_c_string(), VariantDict

### Community 12 - "env_context.rs"
Cohesion: 0.17
Nodes (20): clip_output(), contains_any(), detect_kind(), detects_mysql_from_output(), detects_psql_from_title(), empty_hints_still_shell_when_shell_known(), EnvKind, file_uri_to_path() (+12 more)

### Community 14 - "LaunchOptions"
Cohesion: 0.33
Nodes (9): LaunchOptions, normalize_working_directory(), path_to_string(), Option, PathBuf, Result, String, store() (+1 more)

### Community 15 - "ElTerminalExtension"
Cohesion: 0.42
Nodes (3): FileInfo, MenuItem, ElTerminalExtension

### Community 16 - "test-desktop-integration.sh"
Cohesion: 0.90
Nodes (4): bad(), ok(), run_cwd_case(), test-desktop-integration.sh script

## Knowledge Gaps
- **11 isolated node(s):** `Features`, `Shortcuts`, `Dependencies`, `Build & run`, `CLI` (+6 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppSettings` connect `terminal_tab.rs` to `hit_interactive_child`, `Rc`, `Tab`, `build_ui`?**
  _High betweenness centrality (0.142) - this node is a cross-community bridge._
- **Why does `TerminalContext` connect `env_context.rs` to `Rc`, `build_ui`?**
  _High betweenness centrality (0.109) - this node is a cross-community bridge._
- **Why does `AppState` connect `Tab` to `terminal_tab.rs`, `Rc`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `Features`, `Shortcuts`, `Dependencies` to the rest of the system?**
  _11 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `terminal_tab.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09743589743589744 - nodes in this community are weakly interconnected._
- **Should `Rc` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `Terminal Emulator` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._