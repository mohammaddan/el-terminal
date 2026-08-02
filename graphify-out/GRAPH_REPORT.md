# Graph Report - terminal-emulator  (2026-08-03)

## Corpus Check
- 9 files · ~4,194 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 119 nodes · 221 edges · 14 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

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
- [[_COMMUNITY_Rc|Rc]]
- [[_COMMUNITY_add_tab|add_tab]]

## God Nodes (most connected - your core abstractions)
1. `AppState` - 24 edges
2. `Tab` - 9 edges
3. `BlurState` - 9 edges
4. `build_ui()` - 8 edges
5. `add_tab()` - 8 edges
6. `close_tab_by_name()` - 8 edges
7. `wire_pane()` - 7 edges
8. `split_active()` - 7 edges
9. `BlurDispatch` - 7 edges
10. `close_pane_by_id()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `Tab` --references--> `Pane`  [EXTRACTED]
  src/app.rs → src/app.rs  _Bridges community 3 → community 9_
- `wire_pane()` --references--> `Terminal`  [EXTRACTED]
  src/app.rs →   _Bridges community 3 → community 13_
- `AppState` --references--> `Tab`  [EXTRACTED]
  src/app.rs → src/app.rs  _Bridges community 9 → community 5_
- `active_terminal()` --references--> `AppState`  [EXTRACTED]
  src/app.rs → src/app.rs  _Bridges community 5 → community 3_
- `add_tab()` --references--> `AppState`  [EXTRACTED]
  src/app.rs → src/app.rs  _Bridges community 5 → community 14_

## Import Cycles
- None detected.

## Communities (14 total, 0 thin omitted)

### Community 0 - "terminal_tab.rs"
Cohesion: 0.26
Nodes (14): Path, RGBA, apply_palette(), copy_selection(), create_terminal(), default_title(), parse_rgba(), paste_clipboard() (+6 more)

### Community 1 - "tab_bar.rs"
Cohesion: 0.36
Nodes (8): build_menu_button(), build_new_tab_button(), build_status_dot(), build_tab_pill(), build_window_controls(), Button, GtkBox, Label

### Community 2 - "attach_context_menu"
Cohesion: 0.33
Nodes (6): Fn, attach_context_menu(), install_window_actions(), ApplicationWindow, IsA, Widget

### Community 3 - "Rc"
Cohesion: 0.48
Nodes (6): active_terminal(), focus_pane_terminal(), Pane, Option, Terminal, select_tab_by_name()

### Community 4 - "Terminal Emulator"
Cohesion: 0.33
Nodes (5): Build & run, Dependencies, Features, Layout, Terminal Emulator

### Community 5 - "Tab"
Cohesion: 0.29
Nodes (8): Cell, AppState, focus_pane_in_tab(), install_drag(), ApplicationWindow, GtkBox, RefCell, Stack

### Community 6 - "build_ui"
Cohesion: 0.38
Nodes (7): Application, build_ui(), install_actions(), install_menu(), install_shortcuts(), install_window_controls(), Button

### Community 7 - "blur.rs"
Cohesion: 0.14
Nodes (24): Connection, Dispatch, Event, EventQueue, ExtBackgroundEffectManagerV1, ExtBackgroundEffectSurfaceV1, GlobalListContents, OrgKdeKwinBlur (+16 more)

### Community 8 - "app.rs"
Cohesion: 0.31
Nodes (9): Context, DrawingArea, ImageSurface, build_chrome_background(), build_noise_surface(), hash2(), NoiseCache, rounded_rect() (+1 more)

### Community 9 - "hit_interactive_child"
Cohesion: 0.29
Nodes (7): hit_interactive_child(), IsA, Label, String, Widget, Tab, Vec

### Community 13 - "Rc"
Cohesion: 0.52
Nodes (7): close_active_pane(), close_active_tab(), close_pane_by_id(), close_tab_by_name(), reset_last_tab(), Rc, wire_pane()

### Community 14 - "add_tab"
Cohesion: 0.40
Nodes (6): Orientation, Paned, add_tab(), balance_paned(), next_id(), split_active()

## Knowledge Gaps
- **4 isolated node(s):** `Features`, `Dependencies`, `Build & run`, `Layout`
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `Tab` to `Rc`, `build_ui`, `hit_interactive_child`, `Rc`, `add_tab`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Why does `Tab` connect `hit_interactive_child` to `Rc`, `Tab`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._
- **What connects `Features`, `Dependencies`, `Build & run` to the rest of the system?**
  _4 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `blur.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13538461538461538 - nodes in this community are weakly interconnected._