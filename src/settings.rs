use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const FONT_SIZE_MIN: u32 = 8;
pub const FONT_SIZE_MAX: u32 = 32;
pub const FONT_SIZE_DEFAULT: u32 = 11;
/// Default in-shell Ask prefix. Type `?? how do I …` then Enter.
pub const ASK_PREFIX_DEFAULT: &str = "??";

pub const WINDOW_RADIUS_MIN: f64 = 0.0;
pub const WINDOW_RADIUS_MAX: f64 = 32.0;
pub const TAB_RADIUS_MIN: f64 = 0.0;
pub const TAB_RADIUS_MAX: f64 = 999.0;

fn default_ask_prefix() -> String {
    ASK_PREFIX_DEFAULT.into()
}

fn default_ask_share_terminal_context() -> bool {
    false
}

fn default_theme_style() -> ThemeStyle {
    ThemeStyle::preset(&default_theme_id())
}

/// Named theme pack loaded from a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreset {
    pub id: String,
    pub label: String,
    /// When true, this preset is the fallback/default theme.
    #[serde(default)]
    pub default: bool,
    #[serde(flatten)]
    pub style: ThemeStyle,
}

/// Active theme appearance: UI tokens, chrome, tabs, and VTE palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStyle {
    pub window_radius: f64,
    pub window_border: [f64; 4],
    pub chrome_fill: [f64; 4],
    pub tab_radius: f64,
    pub tab_border: String,
    pub tab_border_active: String,
    pub bg_glass: String,
    pub bg_tab: String,
    pub fg_primary: String,
    pub fg_muted: String,
    pub accent_green: String,
    pub accent_blue: String,
    pub accent_purple: String,
    pub border_subtle: String,
    pub popover_bg: String,
    pub settings_bg: String,
    pub ask_panel_bg: String,
    pub ask_prompt_bg: String,
    pub danger: String,
    pub warning: String,
    pub terminal_fg: String,
    pub terminal_bg: [f32; 4],
    pub palette: [String; 16],
}

impl Default for ThemeStyle {
    fn default() -> Self {
        Self::preset(&default_theme_id())
    }
}

impl ThemeStyle {
    pub fn preset(theme_id: &str) -> Self {
        theme_by_id(theme_id)
            .or_else(|| theme_catalog().first().cloned())
            .expect("no themes loaded; add JSON files under themes/")
            .style
    }

    pub fn normalize(&mut self) {
        self.window_radius = self
            .window_radius
            .clamp(WINDOW_RADIUS_MIN, WINDOW_RADIUS_MAX);
        self.tab_radius = self.tab_radius.clamp(TAB_RADIUS_MIN, TAB_RADIUS_MAX);
        for channel in &mut self.window_border {
            *channel = channel.clamp(0.0, 1.0);
        }
        for channel in &mut self.chrome_fill {
            *channel = channel.clamp(0.0, 1.0);
        }
        for channel in &mut self.terminal_bg {
            *channel = channel.clamp(0.0, 1.0);
        }
        if self.tab_border.trim().is_empty() {
            self.tab_border = "transparent".into();
        }
        if self.tab_border_active.trim().is_empty() {
            self.tab_border_active = self.accent_green.clone();
        }
    }

    /// GTK CSS fragment: `@define-color` tokens plus radius/border overrides.
    pub fn to_css(&self) -> String {
        let r = self.window_radius;
        let tr = self.tab_radius;
        format!(
            r#"@define-color bg_glass {bg_glass};
@define-color bg_tab {bg_tab};
@define-color fg_primary {fg_primary};
@define-color fg_muted {fg_muted};
@define-color accent_green {accent_green};
@define-color accent_blue {accent_blue};
@define-color accent_purple {accent_purple};
@define-color border_subtle {border_subtle};
@define-color popover_bg {popover_bg};
@define-color settings_bg {settings_bg};
@define-color ask_panel_bg {ask_panel_bg};
@define-color ask_prompt_bg {ask_prompt_bg};
@define-color danger {danger};
@define-color warning {warning};
@define-color tab_border {tab_border};
@define-color tab_border_active {tab_border_active};

.window-chrome {{
  border-radius: {r}px;
}}

.top-bar {{
  border-top-left-radius: {r}px;
  border-top-right-radius: {r}px;
}}

.terminal-host {{
  border-bottom-left-radius: {r}px;
  border-bottom-right-radius: {r}px;
}}

.ask-panel {{
  border-bottom-right-radius: {r}px;
}}

.settings-dialog {{
  border-radius: {r}px;
}}

.tab-pill {{
  border-radius: {tr}px;
  border-color: @tab_border;
}}

.tab-pill.active {{
  border-color: @tab_border_active;
}}
"#,
            bg_glass = self.bg_glass,
            bg_tab = self.bg_tab,
            fg_primary = self.fg_primary,
            fg_muted = self.fg_muted,
            accent_green = self.accent_green,
            accent_blue = self.accent_blue,
            accent_purple = self.accent_purple,
            border_subtle = self.border_subtle,
            popover_bg = self.popover_bg,
            settings_bg = self.settings_bg,
            ask_panel_bg = self.ask_panel_bg,
            ask_prompt_bg = self.ask_prompt_bg,
            danger = self.danger,
            warning = self.warning,
            tab_border = self.tab_border,
            tab_border_active = self.tab_border_active,
        )
    }
}

/// Themes from system/bundled `themes/*.json`, plus user overrides under
/// `~/.config/el-terminal/themes/`. Same `id` from the user dir wins.
pub fn theme_catalog() -> &'static [ThemePreset] {
    static CATALOG: OnceLock<Vec<ThemePreset>> = OnceLock::new();
    CATALOG.get_or_init(load_theme_catalog).as_slice()
}

/// Id of the theme marked `"default": true`, else the first loaded theme.
pub fn default_theme_id() -> String {
    theme_catalog()
        .iter()
        .find(|t| t.default)
        .or_else(|| theme_catalog().first())
        .map(|t| t.id.clone())
        .expect("no themes loaded; add JSON files under themes/")
}

pub fn theme_by_id(theme_id: &str) -> Option<ThemePreset> {
    theme_catalog()
        .iter()
        .find(|t| t.id == theme_id)
        .cloned()
}

pub fn theme_ids() -> Vec<String> {
    theme_catalog().iter().map(|t| t.id.clone()).collect()
}

pub fn theme_labels() -> Vec<String> {
    theme_catalog().iter().map(|t| t.label.clone()).collect()
}

fn load_theme_catalog() -> Vec<ThemePreset> {
    let mut presets = Vec::new();

    for dir in bundled_themes_dirs() {
        load_themes_from_dir(&dir, &mut presets);
    }
    load_themes_from_dir(&user_themes_dir(), &mut presets);

    if presets.is_empty() {
        panic!(
            "no themes loaded; looked in {:?} and {}",
            bundled_themes_dirs(),
            user_themes_dir().display()
        );
    }

    // Default-marked theme first for stable dropdown ordering.
    if let Some(idx) = presets.iter().position(|t| t.default) {
        presets.swap(0, idx);
    }

    presets
}

fn load_themes_from_dir(dir: &Path, presets: &mut Vec<ThemePreset>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    for path in paths {
        if let Some(preset) = load_theme_file(&path) {
            upsert_theme(presets, preset);
        }
    }
}

fn upsert_theme(presets: &mut Vec<ThemePreset>, preset: ThemePreset) {
    if let Some(existing) = presets.iter_mut().find(|t| t.id == preset.id) {
        *existing = preset;
    } else {
        presets.push(preset);
    }
}

fn load_theme_file(path: &Path) -> Option<ThemePreset> {
    let raw = fs::read_to_string(path)
        .map_err(|e| eprintln!("el-terminal: failed to read {}: {e}", path.display()))
        .ok()?;
    let mut preset: ThemePreset = serde_json::from_str(&raw)
        .map_err(|e| eprintln!("el-terminal: invalid theme {}: {e}", path.display()))
        .ok()?;
    if preset.id.trim().is_empty() {
        preset.id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("theme")
            .to_string();
    }
    if preset.label.trim().is_empty() {
        preset.label = preset.id.clone();
    }
    preset.style.normalize();
    Some(preset)
}

/// Candidate dirs for shipped themes (later dirs override earlier on same `id`).
fn bundled_themes_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    dirs.push(PathBuf::from("/usr/share/el-terminal/themes"));
    dirs.push(PathBuf::from("/usr/local/share/el-terminal/themes"));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("../share/el-terminal/themes"));
            dirs.push(parent.join("themes"));
        }
    }

    // Dev builds: repo `themes/` wins over system installs.
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("themes"));

    dirs
}

pub fn user_themes_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("el-terminal")
        .join("themes")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub font_size: u32,
    pub font_family: String,
    pub llm_endpoint: String,
    pub llm_api_key: String,
    pub llm_model: String,
    /// Prefix typed in the shell to Ask AI (e.g. `??` or `#?`).
    #[serde(default = "default_ask_prefix")]
    pub ask_prefix: String,
    /// When true, Ask includes cwd / recent output / session hints in the LLM request.
    #[serde(default = "default_ask_share_terminal_context")]
    pub ask_share_terminal_context: bool,
    /// Active theme style (UI + chrome + VTE). Missing on old configs → preset for `theme`.
    #[serde(default = "default_theme_style")]
    pub style: ThemeStyle,
}

impl Default for AppSettings {
    fn default() -> Self {
        let theme = default_theme_id();
        Self {
            style: ThemeStyle::preset(&theme),
            theme,
            font_size: FONT_SIZE_DEFAULT,
            font_family: "JetBrains Mono".into(),
            llm_endpoint: "https://api.openai.com/v1".into(),
            llm_api_key: String::new(),
            llm_model: "gpt-4o-mini".into(),
            ask_prefix: ASK_PREFIX_DEFAULT.into(),
            ask_share_terminal_context: false,
        }
    }
}

impl AppSettings {
    pub fn clamp_font_size(size: u32) -> u32 {
        size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
    }

    pub fn normalize(&mut self) {
        self.font_size = Self::clamp_font_size(self.font_size);
        if theme_by_id(&self.theme).is_none() {
            self.theme = default_theme_id();
        }
        if self.font_family.trim().is_empty() {
            self.font_family = "JetBrains Mono".into();
        }
        self.llm_endpoint = self.llm_endpoint.trim_end_matches('/').to_string();
        if self.llm_endpoint.is_empty() {
            self.llm_endpoint = "https://api.openai.com/v1".into();
        }
        if self.llm_model.trim().is_empty() {
            self.llm_model = "gpt-4o-mini".into();
        }
        let prefix = self.ask_prefix.trim();
        if prefix.is_empty() {
            self.ask_prefix = ASK_PREFIX_DEFAULT.into();
        } else {
            self.ask_prefix = prefix.to_string();
        }
        self.style.normalize();
    }

    /// Apply a named theme preset, replacing `style` with that pack.
    pub fn apply_theme_preset(&mut self, theme_id: &str) {
        let id = if theme_by_id(theme_id).is_some() {
            theme_id.to_string()
        } else {
            default_theme_id()
        };
        self.theme = id.clone();
        self.style = ThemeStyle::preset(&id);
        self.style.normalize();
    }

    pub fn font_description(&self) -> String {
        let family = self.font_family.trim();
        let size = self.font_size;
        format!("{family} {size}, Fira Code {size}, Cascadia Code {size}, monospace {size}")
    }

    pub fn llm_configured(&self) -> bool {
        !self.llm_endpoint.is_empty() && !self.llm_api_key.trim().is_empty()
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("el-terminal")
            .join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let had_style = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .map(|v| v.get("style").is_some())
            .unwrap_or(false);

        let mut settings = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        };

        // Old configs without `style`: seed from the selected theme id.
        if !had_style {
            settings.style = ThemeStyle::preset(&settings.theme);
        }

        settings.normalize();
        settings
    }

    pub fn save(&self) -> Result<(), String> {
        let mut settings = self.clone();
        settings.normalize();
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        fs::write(&path, raw).map_err(|e| e.to_string())
    }
}
