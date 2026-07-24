use std::path::PathBuf;

use gtk::gio;
use gtk::prelude::*;

const MIN_FONT_SIZE: f64 = 6.0;
const MAX_FONT_SIZE: f64 = 72.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub(crate) struct AppSettings {
    pub(crate) font: FontSettings,
    pub(crate) theme: Theme,
}

#[derive(Debug, Clone)]
pub(crate) struct FontSettings {
    pub(crate) family: String,
    pub(crate) size: f64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font: FontSettings::default(),
            theme: Theme::System,
        }
    }
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "monospace".to_owned(),
            size: 12.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsWarning {
    ReadFailed,
    InvalidToml,
    InvalidFontFamily,
    InvalidFontSize,
    InvalidTheme,
}

impl SettingsWarning {
    pub(crate) fn user_message(self) -> &'static str {
        match self {
            Self::ReadFailed => {
                "Die Konfiguration konnte nicht gelesen werden; Standardwerte werden verwendet."
            }
            Self::InvalidToml => {
                "Die Konfiguration ist kein gültiges TOML; Standardwerte werden verwendet."
            }
            Self::InvalidFontFamily => {
                "Die konfigurierte Schriftfamilie ist ungültig; Monospace wird verwendet."
            }
            Self::InvalidFontSize => {
                "Die konfigurierte Schriftgröße ist ungültig; 12 pt werden verwendet."
            }
            Self::InvalidTheme => {
                "Das konfigurierte Farbschema ist ungültig; die Systemeinstellung wird verwendet."
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SettingsLoadOutcome {
    pub(crate) settings: AppSettings,
    pub(crate) warnings: Vec<SettingsWarning>,
}

impl AppSettings {
    /// Loads `$XDG_CONFIG_HOME/iv/config.toml` without blocking the GTK main context.
    pub(crate) async fn load_async() -> SettingsLoadOutcome {
        let file = gio::File::for_path(Self::config_path());
        match file.load_contents_future().await {
            Ok((contents, _etag)) => parse_settings(&contents),
            Err(error) if error.kind::<gio::IOErrorEnum>() == Some(gio::IOErrorEnum::NotFound) => {
                SettingsLoadOutcome::default()
            }
            Err(_) => SettingsLoadOutcome {
                warnings: vec![SettingsWarning::ReadFailed],
                ..SettingsLoadOutcome::default()
            },
        }
    }

    fn config_path() -> PathBuf {
        gtk::glib::user_config_dir().join("iv").join("config.toml")
    }
}

fn parse_settings(contents: &[u8]) -> SettingsLoadOutcome {
    let Ok(contents) = std::str::from_utf8(contents) else {
        return SettingsLoadOutcome {
            warnings: vec![SettingsWarning::InvalidToml],
            ..SettingsLoadOutcome::default()
        };
    };
    let Ok(document) = contents.parse::<toml::Value>() else {
        return SettingsLoadOutcome {
            warnings: vec![SettingsWarning::InvalidToml],
            ..SettingsLoadOutcome::default()
        };
    };

    let mut outcome = SettingsLoadOutcome::default();

    if let Some(font) = document.get("font") {
        if let Some(font) = font.as_table() {
            parse_font_settings(font, &mut outcome);
        } else {
            outcome.warnings.extend([
                SettingsWarning::InvalidFontFamily,
                SettingsWarning::InvalidFontSize,
            ]);
        }
    }

    if let Some(appearance) = document.get("appearance") {
        if let Some(appearance) = appearance.as_table() {
            if let Some(theme) = appearance.get("theme") {
                match theme.as_str() {
                    Some("system") => outcome.settings.theme = Theme::System,
                    Some("light") => outcome.settings.theme = Theme::Light,
                    Some("dark") => outcome.settings.theme = Theme::Dark,
                    _ => outcome.warnings.push(SettingsWarning::InvalidTheme),
                }
            }
        } else {
            outcome.warnings.push(SettingsWarning::InvalidTheme);
        }
    }

    outcome
}

fn parse_font_settings(
    font: &toml::map::Map<String, toml::Value>,
    outcome: &mut SettingsLoadOutcome,
) {
    if let Some(family) = font.get("family") {
        match family.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => outcome.settings.font.family = value.to_owned(),
            _ => outcome.warnings.push(SettingsWarning::InvalidFontFamily),
        }
    }

    if let Some(size) = font.get("size") {
        match size
            .as_float()
            .or_else(|| size.as_integer().map(|value| value as f64))
        {
            Some(value)
                if value.is_finite() && (MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&value) =>
            {
                outcome.settings.font.size = value;
            }
            _ => outcome.warnings.push(SettingsWarning::InvalidFontSize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_settings_are_parsed() {
        let outcome = parse_settings(
            br#"
                [font]
                family = "JetBrains Mono"
                size = 14

                [appearance]
                theme = "dark"
            "#,
        );

        assert_eq!(outcome.settings.font.family, "JetBrains Mono");
        assert_eq!(outcome.settings.font.size, 14.0);
        assert_eq!(outcome.settings.theme, Theme::Dark);
        assert!(outcome.warnings.is_empty());
    }

    #[test]
    fn invalid_fields_fall_back_individually() {
        let outcome = parse_settings(
            br#"
                [font]
                family = ""
                size = 200

                [appearance]
                theme = "blue"
            "#,
        );

        assert_eq!(outcome.settings.font.family, "monospace");
        assert_eq!(outcome.settings.font.size, 12.0);
        assert_eq!(outcome.settings.theme, Theme::System);
        assert_eq!(
            outcome.warnings,
            [
                SettingsWarning::InvalidFontFamily,
                SettingsWarning::InvalidFontSize,
                SettingsWarning::InvalidTheme,
            ]
        );
    }

    #[test]
    fn malformed_or_non_utf8_toml_uses_defaults() {
        for contents in [b"[font\n".as_slice(), b"\xff\xfe".as_slice()] {
            let outcome = parse_settings(contents);
            assert_eq!(outcome.settings.font.size, 12.0);
            assert_eq!(outcome.warnings, [SettingsWarning::InvalidToml]);
        }
    }

    #[test]
    fn integer_and_boundary_font_sizes_are_valid() {
        for size in [MIN_FONT_SIZE, MAX_FONT_SIZE] {
            let source = format!("[font]\nsize = {size}");
            let outcome = parse_settings(source.as_bytes());
            assert_eq!(outcome.settings.font.size, size);
            assert!(outcome.warnings.is_empty());
        }
    }

    #[test]
    fn invalid_section_types_are_reported() {
        let outcome = parse_settings(b"font = true\nappearance = 4");

        assert_eq!(outcome.settings.font.family, "monospace");
        assert_eq!(outcome.settings.font.size, 12.0);
        assert_eq!(outcome.settings.theme, Theme::System);
        assert_eq!(
            outcome.warnings,
            [
                SettingsWarning::InvalidFontFamily,
                SettingsWarning::InvalidFontSize,
                SettingsWarning::InvalidTheme,
            ]
        );
    }
}
