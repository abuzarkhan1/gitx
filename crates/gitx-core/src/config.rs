use serde::{Deserialize, Serialize};

/// GitX configuration contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub identity: IdentityConfig,
}

/// Configuration for identity normalization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Mapping of raw identities (name/email) to normalized names.
    pub user_mappings: std::collections::HashMap<String, String>,
}
