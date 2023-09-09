use std::{sync::Arc, net::ToSocketAddrs};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ssh_stream::Client, config_options::SSHConfig};

pub enum StreamType {
    SSHTunnel,
    TCPStream,
}


pub trait AsyncReadWrite: tokio::io::AsyncRead + tokio::io::AsyncWrite {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + ?Sized> AsyncReadWrite for T {}

pub async fn get_stream<S: AsyncRead + AsyncWrite + Unpin + 'static, >(
    stream_type: StreamType,
    ip_addres: String,
    ssh_config: Option<SSHConfig>,
) -> Option<Box<dyn AsyncReadWrite  + Unpin>> {
    match stream_type {
        StreamType::SSHTunnel => {
            let overall_config = ssh_config.expect("ssh config must exist to use ssh stream");
            let config = overall_config.russh_config;
            let config = Arc::new(config);
            let sh = Client {};

            // let mut agent = russh_keys::agent::client::AgentClient::connect_env().await?;
            // agent.add_identity(&key, &[]).await?;
            let mut session =
                russh::client::connect(config, overall_config.ssh_server_address, sh).await.unwrap();
            session
                .authenticate_password(&overall_config.username, &overall_config.password)
                .await
                .unwrap();
            let mut host = ip_addres.to_socket_addrs().unwrap();
            let address = host.next().unwrap();
            let channel = session
                .channel_open_direct_tcpip(address.ip().to_string(),address.port().to_be() as u32, "127.0.0.1", 51111)
                .await
                .unwrap();

            let stream = channel.into_stream();

            Some(Box::new(stream) as Box<dyn AsyncReadWrite  + Unpin>)
        }
        StreamType::TCPStream => match tokio::net::TcpStream::connect(ip_addres).await {
            Err(e) => {
                println!("{}", e);
                None
            }
            Ok(s) => Some(Box::new(s) as Box<dyn AsyncReadWrite  + Unpin>),
        },
    }
}
