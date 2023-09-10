# Simple Proxy Server in Rust 🦀

<div align="center">
  <img src="https://www.rust-lang.org/logos/rust-logo-512x512.png" width="128px"/>
</div>

---

## 🌟 Features

- 📡 Handles both HTTP and HTTPS requests.
- 🚀 High performance due to non-blocking I/O.
- 🛠️ Easy to set up and use.
- ⚙️ Highly configurable.
- Allow Tunneling over SSH
  
## 🏁 Getting Started

Clone the repo:

```bash
git clone https://github.com/Irvingouj/rusty-proxy.git
```

Navigate into the directory:

```bash
cd simple-proxy-rust
```

Run the project:

```bash
cargo run
```

## 🎯 How to Use

Run the server:

```bash
cargo run
```

Test the proxy with curl:

```bash
curl -x http://localhost:3000 -k https://www.google.com
```

## Example Configuration
```
[server]
host = "127.0.0.1"
port = 5123

[ssh_config]
username = "your-ssh-username"
password = "your-ssh-password"
ssh_server_address = "ssh-server-address"
ssh_server_port = 22
russh_config = 30

```

if ssh_config is present, then it will use ssh tunneling. If you just want to use a simple proxy, only fill up server.

## 💻 Requirements

- Rust 2018 edition or later.
  
## 🤝 Contributing

Feel free to submit pull requests, create issues, or ask questions. We're open to all kinds of contributions!

## 📝 License

MIT License - see [`LICENSE`](LICENSE) for details.


Made with 💖 and 🦀
