use std::path::PathBuf;

use serde::Deserialize;

/// Application settings, loaded from TOML.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct AppSettings {
    /// Font settings.
    #[serde(default)]
    pub(crate) font: FontSettings,
    /// Appearance settings.
    #[serde(default)]
    pub(crate) appearance: AppearanceSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FontSettings {
    /// Font family name (e.g. "monospace", "Fira Code", "JetBrains Mono").
    #[serde(default = "default_font_family")]
    pub(crate) family: String,
    /// Base font size in points.
    #[serde(default = "default_font_size")]
    pub(crate) size: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AppearanceSettings {
    /// Color scheme: "system", "light", or "dark".
    #[serde(default = "default_theme")]
    pub(crate) theme: String,
}

fn default_font_family() -> String {
    "monospace".to_owned()
}

fn default_font_size() -> f64 {
    12.0
}

fn default_theme() -> String {
    "system".to_owned()
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

impl AppSettings {
    /// Load settings from `~/.config/iv/config.toml`. Returns defaults if the
    /// file does not exist or cannot be parsed.
    pub(crate) fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|err| {
                eprintln!("IV: Ungültige Konfiguration in {}: {err}", path.display());
                Self::default()
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(err) => {
                eprintln!("IV: Konnte Konfiguration nicht lesen: {err}");
                Self::default()
            }
        }
    }

    /// Path to the config file: `$XDG_CONFIG_HOME/iv/config.toml`, falling
    /// back to `~/.config/iv/config.toml`.
    fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("iv").join("config.toml")
    }
}
