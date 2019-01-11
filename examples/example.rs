use laminar::config::SocketConfig;
use std::env;
use std::io::stdin;
use std::net::SocketAddr;

fn main() -> Result<(), Box<std::error::Error>> {
    let addr = "127.0.0.1:9000";
//    let socket = LaminarSocket::bind(addr, SocketConfig::default())?;
    Ok(())
}
