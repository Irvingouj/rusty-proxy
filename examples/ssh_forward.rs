use anyhow::Result;
use async_trait::async_trait;
use russh::*;
use russh_keys::*;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
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

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init()?;
    let config = russh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};

    // let mut agent = russh_keys::agent::client::AgentClient::connect_env().await?;
    // agent.add_identity(&key, &[]).await?;
    let mut session = russh::client::connect(config, ("192.168.50.194", 22), sh).await?;
    session
        .authenticate_password("irving", "990225ojy")
        .await
        .unwrap();

    let mut channel = session
        .channel_open_direct_tcpip("www.example.com", 80, "127.0.0.1", 51241)
        .await
        .unwrap();

    let mut stream = channel.into_stream();
    let request = b"GET / HTTP/1.1\r\nHost: www.example.com\r\nConnection: close\r\n\r\n";
    stream.write(request).await.unwrap();

    let mut buf = Vec::<u8>::new();
    let mut counter = 0;
    loop {
        if let Ok(bytes_read) = stream.read_to_end(&mut buf).await {
            println!(
                "msg red = {:?}, ascii = {:?}",
                &buf[..bytes_read],
                String::from_utf8_lossy(&buf[..bytes_read])
            );
            buf.clear();
            counter +=1;
            if counter >=10{
                panic!()
            }
        } else {
            break;
        }
    }

    Ok(())
}
