use crate::identity::IdentityMapping;
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
    pub identity: IdentityConfig,
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
    /// Optional directory for the persisted index. When set (or when the
    /// `GITX_CACHE_DIR` env var is set), the index lives there instead of
    /// inside `.git/` (docs/16 §6 cache location configurable).
    pub cache_dir: Option<String>,
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

/// Contributor identity configuration (docs/05 §3): explicit user mappings
/// from raw email to canonical display name. Never merge on weak guesses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct IdentityConfig {
    pub mappings: Vec<IdentityMapping>,
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
            cache_dir: None,
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
    /// The `GITX_CONFIG` env var overrides it (docs/16 §4 env-var precedence).
    pub fn default_path() -> Option<PathBuf> {
        if let Some(p) = std::env::var_os("GITX_CONFIG") {
            return Some(PathBuf::from(p));
        }
        Self::platform_default_path()
    }

    fn platform_default_path() -> Option<PathBuf> {
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

    /// The configured cache directory: `[index] cache_dir` in the config, or
    /// the `GITX_CACHE_DIR` env var (docs/16 §6).
    pub fn cache_dir(&self) -> Option<String> {
        self.index
            .cache_dir
            .clone()
            .or_else(|| std::env::var("GITX_CACHE_DIR").ok())
    }

    /// Merge `other` over `self` for repository-local layering (docs/16
    /// §4–§5). Fields that `other` leaves at their defaults keep `self`'s
    /// values; fields explicitly set in `other` win. This works because all
    /// defaults are stable constants.
    pub fn merge(&mut self, other: Config) {
        let base = Config::default();
        if other.general.default_limit != base.general.default_limit {
            self.general.default_limit = other.general.default_limit;
        }
        if other.general.color != base.general.color {
            self.general.color = other.general.color;
        }
        if other.index.enabled != base.index.enabled {
            self.index.enabled = other.index.enabled;
        }
        if other.index.auto_refresh != base.index.auto_refresh {
            self.index.auto_refresh = other.index.auto_refresh;
        }
        if other.index.cache_dir.is_some() {
            self.index.cache_dir = other.index.cache_dir;
        }
        if other.analysis.hotspot_change_frequency_weight
            != base.analysis.hotspot_change_frequency_weight
            || other.analysis.hotspot_recent_churn_weight
                != base.analysis.hotspot_recent_churn_weight
            || other.analysis.hotspot_bug_fix_weight != base.analysis.hotspot_bug_fix_weight
            || other.analysis.hotspot_ownership_weight != base.analysis.hotspot_ownership_weight
            || other.analysis.hotspot_complexity_weight != base.analysis.hotspot_complexity_weight
        {
            self.analysis = other.analysis;
        }
        if other.ui.theme != base.ui.theme {
            self.ui.theme = other.ui.theme;
        }
        if other.ui.vim_keys != base.ui.vim_keys {
            self.ui.vim_keys = other.ui.vim_keys;
        }
        if other.search.case_sensitive != base.search.case_sensitive {
            self.search.case_sensitive = other.search.case_sensitive;
        }
        if !other.identity.mappings.is_empty() {
            self.identity.mappings = other.identity.mappings;
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
