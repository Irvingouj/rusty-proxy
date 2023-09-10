mod config_options;
mod errors;
mod ssh_stream;
mod streams;

use config_options::{get_config, Config};
use errors::ProxyError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

use crate::streams::get_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_module_level("rusty-proxy", log::LevelFilter::Debug)
        .with_module_level("russh", log::LevelFilter::Debug)
        .init()?;
    let config = get_config(None);
    start_server_loop(&config).await
}

async fn start_server_loop(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.host, config.server.port).to_string();
    let listener = TcpListener::bind(&addr).await?;
    log::info!("started listening at {}", &addr);
    loop {
        let (mut listening_stream, client_addr) = listener.accept().await?;
        log::debug!("received client : {}", &client_addr);
        let config = config.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 8096];
            match listening_stream.read(&mut buf).await {
                Ok(bytes_read) if bytes_read == 0 => {
                    log::error!("Connection closed");
                    return;
                }
                Ok(bytes_read) => {
                    log::debug!("{}", String::from_utf8_lossy(&buf[0..bytes_read]));

                    match pre_request_hook(&mut buf) {
                        Ok(()) => {}
                        Err(e) => {
                            log::error!("Error on pre-request-hook: {:?}", e);
                            return;
                        }
                    }

                    let (start, end) = match get_url_index(&buf) {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("error on parsing url {}", e);
                            return;
                        }
                    };
                    let url = buf[start..end].to_vec();
                    log::debug!("url = {}", String::from_utf8_lossy(&url));

                    if &buf[0..7] == b"CONNECT" {
                        handle_connect_request(listening_stream, &url, &config)
                            .await
                            .unwrap();
                    } else {
                        handle_http_request(&mut listening_stream, &mut buf, &url, &config)
                            .await
                            .unwrap();
                    }
                }
                Err(e) => {
                    log::error!("Error reading from socket: {}", e);
                }
            }
        });
    }
}

fn pre_request_hook(_buf: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn handle_connect_request(
    mut incoming_stream: TcpStream,
    url_buf: &[u8],
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let url_str = &String::from_utf8_lossy(url_buf).to_string();
    let url_parts: Vec<&str> = url_str.split(':').collect();
    if url_parts.len() != 2 {
        return Err("Invalid URL format".into());
    }
    let host = url_parts[0].trim();
    let port: u16 = url_parts[1].parse().map_err(|_| "Invalid port")?;
    log::debug!("connecting tunnel to {}:{}", host, port);

    incoming_stream.writable().await?;
    incoming_stream
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await?;
    let mut outgoing_stream = get_stream(host,port, config).await.unwrap();
    tokio::io::copy_bidirectional(&mut incoming_stream, &mut outgoing_stream)
        .await
        .unwrap();
    Ok(())
}

async fn handle_http_request(
    incoming_stream: &mut TcpStream,
    buf: &mut [u8],
    url_buf: &[u8],
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Starting to handle HTTP request");

    let url_str = String::from_utf8_lossy(url_buf);
    let parsed_url = Url::parse(&url_str)?;

    let host = parsed_url.host_str().ok_or("Missing host")?;
    let port = parsed_url.port().unwrap_or(80);

    log::debug!("Parsed URL: host={}, port={}", host, port);

    let mut outgoing_stream = get_stream(&host.to_string(), port, &config).await.unwrap();
    outgoing_stream.write_all(&buf).await?;
    log::debug!("Initial write to outgoing stream complete");

    tokio::io::copy_bidirectional(incoming_stream, &mut outgoing_stream).await?;

    log::debug!("Finishing HTTP request handling");
    Ok(())
}

fn get_url_index(buf: &[u8]) -> Result<(usize, usize), ProxyError> {
    let mut first_space = 0;
    let mut second_space = 0;
    let mut index = 0;

    for byte in buf.iter().copied() {
        if byte == b' ' {
            if first_space == 0 {
                first_space = index;
            } else if second_space == 0 {
                second_space = index;
                return Ok((first_space, second_space));
            }
        }
        index += 1;
    }

    Err(ProxyError::CrlfSequenceNotFoundError)
}
