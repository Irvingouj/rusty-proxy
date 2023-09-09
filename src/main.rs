mod errors;
mod streams;
mod ssh_stream;
use errors::ProxyError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

use crate::ssh_stream::SSHConfig;
use crate::streams::get_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server_loop().await
}

async fn start_server_loop() -> Result<(), Box<dyn std::error::Error>>{
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut listening_stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 8096];
            match listening_stream.read(&mut buf).await {
                Ok(bytes_read) if bytes_read == 0 => {
                    eprintln!("Connection closed");
                    return;
                }
                Ok(bytes_read) => {
                    dbg!(String::from_utf8_lossy(&buf[0..bytes_read]));
                    match pre_request_hook(&mut buf) {
                        Ok(())=>{},
                        Err(e)=>{
                            eprintln!("Error on pre-request-hook: {e}");
                            return;
                        }
                    }

                    let (start, end) = match get_url_index(&buf) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("error on parsing url {}", e);
                            return;
                        }
                    };
                    let url = buf[start..end].to_vec();
                    dbg!(start, end, String::from_utf8_lossy(&url));

                    if &buf[0..7] == b"CONNECT" {
                        handle_connect_request(listening_stream, &url)
                            .await
                            .unwrap();
                    } else {
                        handle_http_request(&mut listening_stream, &mut buf, &url)
                            .await
                            .unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                }
            }
        });
    }
}

fn pre_request_hook(buf:&mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn handle_connect_request(
    mut incoming_stream: TcpStream,
    url_buf: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let url_str = &String::from_utf8_lossy(url_buf).to_string();
    let url_parts: Vec<&str> = url_str.split(':').collect();
    if url_parts.len() != 2 {
        return Err("Invalid URL format".into());
    }
    let host = url_parts[0].trim();
    let port: u16 = url_parts[1].parse().map_err(|_| "Invalid port")?;
    dbg!(host,port);

    incoming_stream.writable().await?;
    incoming_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
    let addr = format!("{}:{}", host, port);
    let mut outgoing_stream = TcpStream::connect(addr).await?;
    tokio::io::copy_bidirectional(&mut incoming_stream, & mut outgoing_stream).await.unwrap();
    dbg!("tunnel is open");

    Ok(())
}

async fn handle_http_request(
    incomming_stream: &mut TcpStream,
    buf: &mut [u8],
    url_buf: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let url_str = &String::from_utf8_lossy(url_buf).to_string();
    let parsed_url = Url::parse(url_str)?;

    // Extract host and port
    let host = parsed_url.host_str().ok_or("Missing host")?;
    let port = parsed_url.port().unwrap_or(80); // Default to 80 if port is missing

    // Create the socket address
    let addr = format!("{}:{}", host, port);
    let mut outcomming_stream = TcpStream::connect(addr).await?;

    outcomming_stream.writable().await?;
    outcomming_stream.write(buf).await?;
    outcomming_stream.readable().await?;
    let size = outcomming_stream.read(buf).await?;
    incomming_stream.writable().await?;
    incomming_stream.write(&buf[..size]).await?;

    return Ok(());
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
