use std::path::{Path, PathBuf};

pub fn default_vault_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join(".config")
        .join("hermes-hub")
        .join("secrets.vault.json")
}
