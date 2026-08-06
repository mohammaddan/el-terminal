use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const FONT_SIZE_MIN: u32 = 8;
pub const FONT_SIZE_MAX: u32 = 32;
pub const FONT_SIZE_DEFAULT: u32 = 11;
/// Default in-shell Ask prefix. Type `?? how do I …` then Enter.
pub const ASK_PREFIX_DEFAULT: &str = "??";

pub const THEME_IDS: &[&str] = &["glass-dark", "nord", "solarized-dark", "light"];
pub const THEME_LABELS: &[&str] = &["Glass Dark", "Nord", "Solarized Dark", "Light"];

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
    ThemeStyle::preset("glass-dark")
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
        Self::preset("glass-dark")
    }
}

impl ThemeStyle {
    pub fn preset(theme_id: &str) -> Self {
        match theme_id {
            "nord" => Self::nord(),
            "solarized-dark" => Self::solarized_dark(),
            "light" => Self::light(),
            _ => Self::glass_dark(),
        }
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

    fn glass_dark() -> Self {
        Self {
            window_radius: 12.0,
            window_border: [1.0, 1.0, 1.0, 0.20],
            chrome_fill: [0.051, 0.059, 0.071, 0.93],
            tab_radius: 999.0,
            tab_border: "transparent".into(),
            tab_border_active: "#3dd68c".into(),
            bg_glass: "alpha(#0d0f12, 0.55)".into(),
            bg_tab: "alpha(#1a1e24, 0.55)".into(),
            fg_primary: "#e6e8eb".into(),
            fg_muted: "#8b929a".into(),
            accent_green: "#3dd68c".into(),
            accent_blue: "#6cb6ff".into(),
            accent_purple: "#b794f6".into(),
            border_subtle: "alpha(#ffffff, 0.10)".into(),
            popover_bg: "alpha(#14171c, 0.92)".into(),
            settings_bg: "alpha(#14171c, 0.96)".into(),
            ask_panel_bg: "alpha(#12151a, 0.92)".into(),
            ask_prompt_bg: "alpha(#0d0f12, 0.55)".into(),
            danger: "#e35d6a".into(),
            warning: "#e5c07b".into(),
            terminal_fg: "#e6e8eb".into(),
            terminal_bg: [0.051, 0.059, 0.071, 0.55],
            palette: [
                "#0d0f12".into(),
                "#ff6b6b".into(),
                "#3dd68c".into(),
                "#e5c07b".into(),
                "#6cb6ff".into(),
                "#b794f6".into(),
                "#56b6c2".into(),
                "#e6e8eb".into(),
                "#5c6370".into(),
                "#ff8787".into(),
                "#5eead4".into(),
                "#f0d78c".into(),
                "#89b4ff".into(),
                "#c4b5fd".into(),
                "#67e8f9".into(),
                "#ffffff".into(),
            ],
        }
    }

    fn nord() -> Self {
        Self {
            window_radius: 12.0,
            window_border: [0.847, 0.870, 0.914, 0.22], // #d8dee9
            chrome_fill: [0.180, 0.204, 0.251, 0.93],   // #2e3440
            tab_radius: 999.0,
            tab_border: "transparent".into(),
            tab_border_active: "#88c0d0".into(),
            bg_glass: "alpha(#2e3440, 0.55)".into(),
            bg_tab: "alpha(#3b4252, 0.55)".into(),
            fg_primary: "#d8dee9".into(),
            fg_muted: "#4c566a".into(),
            accent_green: "#a3be8c".into(),
            accent_blue: "#81a1c1".into(),
            accent_purple: "#b48ead".into(),
            border_subtle: "alpha(#d8dee9, 0.12)".into(),
            popover_bg: "alpha(#3b4252, 0.94)".into(),
            settings_bg: "alpha(#3b4252, 0.96)".into(),
            ask_panel_bg: "alpha(#2e3440, 0.94)".into(),
            ask_prompt_bg: "alpha(#2e3440, 0.55)".into(),
            danger: "#bf616a".into(),
            warning: "#ebcb8b".into(),
            terminal_fg: "#d8dee9".into(),
            terminal_bg: [0.180, 0.204, 0.251, 0.55],
            palette: [
                "#3b4252".into(),
                "#bf616a".into(),
                "#a3be8c".into(),
                "#ebcb8b".into(),
                "#81a1c1".into(),
                "#b48ead".into(),
                "#88c0d0".into(),
                "#e5e9f0".into(),
                "#4c566a".into(),
                "#bf616a".into(),
                "#a3be8c".into(),
                "#ebcb8b".into(),
                "#81a1c1".into(),
                "#b48ead".into(),
                "#8fbcbb".into(),
                "#eceff4".into(),
            ],
        }
    }

    fn solarized_dark() -> Self {
        Self {
            window_radius: 12.0,
            window_border: [0.514, 0.580, 0.588, 0.22], // #839496
            chrome_fill: [0.000, 0.169, 0.212, 0.93],   // #002b36
            tab_radius: 999.0,
            tab_border: "transparent".into(),
            tab_border_active: "#2aa198".into(),
            bg_glass: "alpha(#002b36, 0.55)".into(),
            bg_tab: "alpha(#073642, 0.55)".into(),
            fg_primary: "#839496".into(),
            fg_muted: "#586e75".into(),
            accent_green: "#859900".into(),
            accent_blue: "#268bd2".into(),
            accent_purple: "#6c71c4".into(),
            border_subtle: "alpha(#839496, 0.14)".into(),
            popover_bg: "alpha(#073642, 0.94)".into(),
            settings_bg: "alpha(#073642, 0.96)".into(),
            ask_panel_bg: "alpha(#002b36, 0.94)".into(),
            ask_prompt_bg: "alpha(#002b36, 0.55)".into(),
            danger: "#dc322f".into(),
            warning: "#b58900".into(),
            terminal_fg: "#839496".into(),
            terminal_bg: [0.000, 0.169, 0.212, 0.55],
            palette: [
                "#073642".into(),
                "#dc322f".into(),
                "#859900".into(),
                "#b58900".into(),
                "#268bd2".into(),
                "#d33682".into(),
                "#2aa198".into(),
                "#eee8d5".into(),
                "#002b36".into(),
                "#cb4b16".into(),
                "#586e75".into(),
                "#657b83".into(),
                "#839496".into(),
                "#6c71c4".into(),
                "#93a1a1".into(),
                "#fdf6e3".into(),
            ],
        }
    }

    fn light() -> Self {
        Self {
            window_radius: 12.0,
            window_border: [0.102, 0.114, 0.137, 0.18], // #1a1d23
            chrome_fill: [0.961, 0.965, 0.973, 0.95],   // #f5f6f8
            tab_radius: 999.0,
            tab_border: "transparent".into(),
            tab_border_active: "#2f9e6e".into(),
            bg_glass: "alpha(#f5f6f8, 0.85)".into(),
            bg_tab: "alpha(#e6e8eb, 0.75)".into(),
            fg_primary: "#1a1d23".into(),
            fg_muted: "#6b7280".into(),
            accent_green: "#2f9e6e".into(),
            accent_blue: "#3b82c4".into(),
            accent_purple: "#8b6cc7".into(),
            border_subtle: "alpha(#1a1d23, 0.12)".into(),
            popover_bg: "alpha(#ffffff, 0.96)".into(),
            settings_bg: "alpha(#ffffff, 0.98)".into(),
            ask_panel_bg: "alpha(#f5f6f8, 0.96)".into(),
            ask_prompt_bg: "alpha(#e6e8eb, 0.70)".into(),
            danger: "#e35d6a".into(),
            warning: "#b08900".into(),
            terminal_fg: "#1a1d23".into(),
            terminal_bg: [0.961, 0.965, 0.973, 0.85],
            palette: [
                "#1a1d23".into(),
                "#e35d6a".into(),
                "#2f9e6e".into(),
                "#b08900".into(),
                "#3b82c4".into(),
                "#8b6cc7".into(),
                "#2a9d8f".into(),
                "#e6e8eb".into(),
                "#6b7280".into(),
                "#ef7a84".into(),
                "#3dd68c".into(),
                "#e5c07b".into(),
                "#6cb6ff".into(),
                "#b794f6".into(),
                "#56b6c2".into(),
                "#ffffff".into(),
            ],
        }
    }
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
        Self {
            theme: "glass-dark".into(),
            font_size: FONT_SIZE_DEFAULT,
            font_family: "JetBrains Mono".into(),
            llm_endpoint: "https://api.openai.com/v1".into(),
            llm_api_key: String::new(),
            llm_model: "gpt-4o-mini".into(),
            ask_prefix: ASK_PREFIX_DEFAULT.into(),
            ask_share_terminal_context: false,
            style: ThemeStyle::preset("glass-dark"),
        }
    }
}

impl AppSettings {
    pub fn clamp_font_size(size: u32) -> u32 {
        size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
    }

    pub fn normalize(&mut self) {
        self.font_size = Self::clamp_font_size(self.font_size);
        if !THEME_IDS.contains(&self.theme.as_str()) {
            self.theme = "glass-dark".into();
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
        let id = if THEME_IDS.contains(&theme_id) {
            theme_id
        } else {
            "glass-dark"
        };
        self.theme = id.into();
        self.style = ThemeStyle::preset(id);
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
