use async_trait::async_trait;
use russh::{client, ChannelId};
use russh_keys::key::PublicKey;

use crate::config_options::SSHConfig;

pub struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &PublicKey  // Removed explicit lifetime
    ) -> Result<(Self, bool), Self::Error> {
        println!("check_server_key: {:?}", server_public_key);
        Ok((self, true))
    }


    async fn data(
        self,
        channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        println!(
            "data on channel {:?}: {:?}",
            channel,
            std::str::from_utf8(data)
        );
        Ok((self, session))
    }
}


impl SSHConfig {
    pub fn new() -> Self {
        SSHConfig {
            username: "".to_string(),
            password: "".to_string(),
            ssh_server_address: "0.0.0.0:22".to_string(),
            russh_config: russh::client::Config::default(),
        }
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn with_ssh_server_address(mut self, ssh_server_address: String) -> Self {
        self.ssh_server_address = ssh_server_address;
        self
    }

    pub fn with_russh_config(mut self, russh_config: russh::client::Config) -> Self {
        self.russh_config = russh_config;
        self
    }
}