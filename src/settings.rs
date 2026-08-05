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

fn default_ask_prefix() -> String {
    ASK_PREFIX_DEFAULT.into()
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
    }

    pub fn font_description(&self) -> String {
        let family = self.font_family.trim();
        let size = self.font_size;
        format!(
            "{family} {size}, Fira Code {size}, Cascadia Code {size}, monospace {size}"
        )
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
        let mut settings = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        };
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
