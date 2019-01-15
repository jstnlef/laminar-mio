use laminar::config::SocketConfig;
use laminar::error::NetworkError;
use laminar::net::RudpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::env;
use std::io::{self, stdin};
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<NetworkError>> {
    let config = SocketConfig::default();
    let local_address: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let (mut socket, packet_receiver) = RudpSocket::bind(local_address, config).unwrap();

    let thread = thread::spawn(move || socket.start_polling());

    loop {
        let packet = packet_receiver.recv().unwrap();
        println!("{:?}", packet);
    }

    Ok(())
}
