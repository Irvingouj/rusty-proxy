use serde::Deserialize;
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub ssh_config: Option<SSHConfig>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SSHConfig {
    pub username: String,
    pub password: String,
    pub ssh_server_address: String,
    pub ssh_server_port:u16,
    #[serde(with = "russh_config_serde")]
    pub russh_config: russh::client::Config,
}

mod russh_config_serde {
    use russh::client::Config;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(value: &Config, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize only the connection_timeout field
        let timeout_secs = value.inactivity_timeout.map(|d| d.as_secs());
        timeout_secs.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Config, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize only the connection_timeout field
        let timeout_secs: Option<u64> = Option::deserialize(deserializer)?;
        let mut config = Config::default();
        config.inactivity_timeout = timeout_secs.map(Duration::from_secs);

        Ok(config)
    }
}

fn get_default_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_default();
    path.push(".rusty-proxy");
    path.push("config.toml");
    path
}

fn create_default_config() -> Config {
    return Config{
        server: ServerConfig { host: "127.0.0.1".to_string(), port: 5123 },
        ssh_config:None
    };
}

fn write_config_to_file(config: &Config, path: &PathBuf) {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directory");
    }

    let toml_str = toml::to_string(config).expect("Failed to serialize config");
    let mut file = File::create(path).expect("Failed to create config file");
    file.write_all(toml_str.as_bytes())
        .expect("Failed to write to config file");
}


pub fn get_config(path: Option<PathBuf>) -> Config {
    let config_path = path.unwrap_or_else(|| get_default_path());

    let config = if config_path.exists() {
        let config_str = fs::read_to_string(&config_path).expect("Failed to read config file");
        toml::from_str(&config_str).expect("Failed to deserialize config")
    } else {
        let default_config = create_default_config();
        write_config_to_file(&default_config, &config_path);
        default_config
    };

    log::debug!("Using config: {}", serde_json::to_string_pretty(&config).unwrap());

    config
}

impl Clone for SSHConfig {
    fn clone(&self) -> Self {
        let mut russh_config = russh::client::Config::default();
        russh_config.inactivity_timeout = self.russh_config.inactivity_timeout.clone();
        SSHConfig {
            username: self.username.clone(),
            password: self.password.clone(),
            ssh_server_address: self.ssh_server_address.clone(),
            ssh_server_port:self.ssh_server_port.clone(),
            // Handle russh::client::Config cloning manually.
            // If russh::client::Config cannot be cloned, you may need to construct a new default instance or handle this some other way.
            russh_config
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}


pub fn russh_config_from_referece(config :&russh::client::Config) -> Arc<russh::client::Config> {
    return Arc::new(russh::client::Config{
        inactivity_timeout:config.inactivity_timeout,
        ..Default::default()
    });
}