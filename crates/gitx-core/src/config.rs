use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// GitX configuration (docs/16). All fields have defaults so the tool works
/// with zero configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub index: IndexConfig,
    pub analysis: AnalysisConfig,
    pub ui: UiConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub default_limit: usize,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IndexConfig {
    pub enabled: bool,
    pub auto_refresh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisConfig {
    pub hotspot_change_frequency_weight: f64,
    pub hotspot_recent_churn_weight: f64,
    pub hotspot_bug_fix_weight: f64,
    pub hotspot_ownership_weight: f64,
    pub hotspot_complexity_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub vim_keys: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub case_sensitive: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_limit: 50,
            color: "auto".into(),
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_refresh: true,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            hotspot_change_frequency_weight: 0.25,
            hotspot_recent_churn_weight: 0.20,
            hotspot_bug_fix_weight: 0.20,
            hotspot_ownership_weight: 0.15,
            hotspot_complexity_weight: 0.20,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            vim_keys: true,
        }
    }
}

impl Config {
    /// Load configuration from `path` (TOML). Missing file → defaults.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(anyhow::anyhow!("cannot read {}: {e}", path.display())),
        };
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("invalid config {}: {e}", path.display()))
    }

    /// Platform-appropriate user configuration path (docs/16 §2).
    /// `~/.config/gitx/config.toml` on Unix, `%APPDATA%\gitx\config.toml` on Windows.
    pub fn default_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("gitx").join("config.toml"))
        }
        #[cfg(not(windows))]
        {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
                .map(|base| base.join("gitx").join("config.toml"))
        }
    }
}

/// Write a starter config file at `path` (used by `gitx config init`).
pub fn write_example(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let example = toml::to_string_pretty(&Config::default())
        .map_err(|e| anyhow::anyhow!("cannot serialize config: {e}"))?;
    std::fs::write(path, example)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let config = Config::default();
        assert_eq!(config.general.default_limit, 50);
        assert!(config.index.enabled);
        assert!(!config.search.case_sensitive);
        // Weights sum to 1.0 (docs/10 hotspot formula).
        let a = &config.analysis;
        let sum = a.hotspot_change_frequency_weight
            + a.hotspot_recent_churn_weight
            + a.hotspot_bug_fix_weight
            + a.hotspot_ownership_weight
            + a.hotspot_complexity_weight;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn round_trips_through_toml() {
        let config = Config::default();
        let text = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.general.default_limit, 50);
    }

    #[test]
    fn partial_config_file_keeps_defaults_for_missing_sections() {
        let dir = std::env::temp_dir().join(format!("gitx-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, "[general]\ndefault_limit = 7\n").unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.general.default_limit, 7);
        assert!(
            config.index.enabled,
            "missing [index] section should keep defaults"
        );
        assert_eq!(config.ui.theme, "default");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_yields_defaults() {
        let config = Config::load(Path::new("/nonexistent/gitx/config.toml")).unwrap();
        assert_eq!(config.ui.theme, "default");
    }
}
