use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub show_addresses: bool,
    pub theme_manifest_path: PathBuf,
}

impl AppConfig {
    fn config_path() -> Result<PathBuf, Error> {
        Ok(dirs::config_dir()
            .ok_or(Error::GeneralError("failed to retrieve config directory"))?
            .join("aseqmatrix")
            .join("config.toml"))
    }

    fn default_theme_manifest_path() -> PathBuf {
        PathBuf::from("themes/memphis/theme.toml")
    }

    pub fn new() -> Result<AppConfig, Error> {
        if let Ok(config_toml) = &fs::read(Self::config_path()?) {
            Ok(toml::from_slice(config_toml)?)
        } else {
            Ok(AppConfig { show_addresses: false, theme_manifest_path: Self::default_theme_manifest_path() })
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        let config_path = Self::config_path()?;

        fs::create_dir_all(
            config_path
                .parent()
                .ok_or(Error::GeneralError("failed to retrieve parent directory of config manifest path"))?,
        )?;
        fs::write(config_path, &toml::to_vec(self)?)?;

        Ok(())
    }
}
