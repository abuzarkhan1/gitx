use crate::cli::{Cli, ConfigAction};
use crate::commands::print_json;
use anyhow::Context;
use serde_json::json;

/// Resolve the effective config path: `--config` flag, else the platform
/// default (docs/16 §4 precedence: CLI flags win).
pub fn config_path(cli: &Cli) -> Option<std::path::PathBuf> {
    cli.config.clone().or_else(gitx_core::Config::default_path)
}

/// Load the effective configuration (defaults overlaid by the file).
pub fn load_config(cli: &Cli) -> anyhow::Result<gitx_core::Config> {
    match config_path(cli) {
        Some(path) => gitx_core::Config::load(&path)
            .with_context(|| format!("config error at {}", path.display())),
        None => Ok(gitx_core::Config::default()),
    }
}

pub fn config_command(cli: &Cli, action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Show => {
            let path = config_path(cli);
            let config = load_config(cli)?;
            if cli.json {
                return print_json(&json!({
                    "path": path.map(|p| p.display().to_string()),
                    "config": config,
                }));
            }
            if let Some(p) = &path {
                println!("config: {}", p.display());
            } else {
                println!("config: (no default path available on this platform)");
            }
            println!("{}", toml::to_string_pretty(&config).unwrap_or_default());
            Ok(())
        }
        ConfigAction::Init => {
            let path = config_path(cli)
                .context("cannot determine a config path on this platform; use --config <PATH>")?;
            gitx_core::write_example(&path)?;
            if cli.json {
                return print_json(&json!({"path": path.display().to_string(), "written": true}));
            }
            println!("Wrote example config to {}", path.display());
            Ok(())
        }
    }
}
