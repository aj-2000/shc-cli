use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Deserialize, Serialize, Clone, Debug)]

pub struct AppConfig {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub user_id: Option<String>,
}

impl AppConfig {
    pub fn new(config_path: &PathBuf) -> Self {
        if !config_path.exists() {
            let config = AppConfig {
                access_token: None,
                refresh_token: None,
                email: None,
                name: None,
                user_id: None,
            };
            config.save(config_path);
            return config;
        }

        let contents =
            fs::read_to_string(config_path).expect("Something went wrong reading the file");

        let config: AppConfig = toml::from_str(&contents).expect("Could not parse TOML");

        config
    }

    pub fn save(&self, config_path: &PathBuf) {
        let toml = toml::to_string(self).unwrap();
        fs::write(config_path, toml).unwrap();
    }

    // FIXME: not working
    pub fn clear(&mut self, config_path: &PathBuf) {
        self.access_token = None;
        self.refresh_token = None;
        self.email = None;
        self.name = None;
        self.user_id = None;
        self.save(config_path);
    }
}
