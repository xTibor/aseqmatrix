use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub show_addresses: bool,
    pub theme_manifest_path: PathBuf,
}

impl AppConfig {
    fn config_path() -> PathBuf {
        dirs::config_dir().unwrap().join("aseqmatrix").join("config.toml")
    }

    pub fn new() -> AppConfig {
        if let Ok(config_toml) = &fs::read(Self::config_path()) {
            toml::from_slice(config_toml).unwrap()
        } else {
            AppConfig { show_addresses: false, theme_manifest_path: PathBuf::from("themes/memphis/theme.toml") }
        }
    }

    pub fn save(&self) {
        let config_path = Self::config_path();
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(config_path, &toml::to_vec(self).unwrap()).unwrap();
    }
}
