use crate::{
    config_options::{russh_config_from_referece, Config},
    ssh_stream::Client,
};
use std::time::Duration;
use tracing::instrument;

// pub enum StreamType {
//     SSHTunnel,
//     TCPStream,
// }

pub trait AsyncReadWrite: tokio::io::AsyncRead + tokio::io::AsyncWrite {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + ?Sized> AsyncReadWrite for T {}

#[instrument]
pub async fn get_stream(
    target_host:&str,
    target_port:u16,
    config: &Config,
) -> Option<Box<dyn AsyncReadWrite + Unpin + Send>> {
    match config.ssh_config {
        Some(ref ssh_config) => {
            let russh_config = russh_config_from_referece(&ssh_config.russh_config);
            let sh = Client {};

            // let mut agent = russh_keys::agent::client::AgentClient::connect_env().await?;
            // agent.add_identity(&key, &[]).await?;
            let ssh_sever_address = (
                ssh_config.ssh_server_address.clone(),
                ssh_config.ssh_server_port,
            );
            let mut session = russh::client::connect(russh_config, ssh_sever_address, sh)
                .await
                .unwrap();
            session
                .authenticate_password(&ssh_config.username, &ssh_config.password)
                .await
                .unwrap();

            log::debug!("About to open channel,address ");
            let res = tokio::time::timeout(
                Duration::from_secs(10),
                session.channel_open_direct_tcpip(
                    target_host,
                    target_port as u32,
                    "127.0.0.1",
                    51241,
                ),
            )
            .await;
            match res {
                Ok(Ok(channel)) => {
                    log::debug!("Channel opened");
                    let stream = channel.into_stream();
                    log::debug!("using ssh stream");
                    Some(Box::new(stream) as Box<dyn AsyncReadWrite + Unpin + Send>)
                }
                _=>panic!("error")
            }
        }
        None => match tokio::net::TcpStream::connect((target_host.clone(),target_port as u16)).await {
            Err(e) => {
                println!("{}", e);
                None
            }
            Ok(s) => {
                log::debug!("using tcp stream");
                Some(Box::new(s) as Box<dyn AsyncReadWrite + Unpin + Send>)
            }
        },
    }
}
