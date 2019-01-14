use laminar::config::SocketConfig;
use laminar::error::NetworkError;
use laminar::net::RudpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::env;
use std::io::{self, stdin};
use std::net::SocketAddr;
use std::time::Duration;

const RECEIVER: Token = Token(0);

fn main() -> Result<(), Box<NetworkError>> {
    let config = SocketConfig::default();
    let local_address: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let mut socket = RudpSocket::bind(local_address, config).unwrap();

    let poll = Poll::new().unwrap();

    poll.register(&socket, RECEIVER, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut events = Events::with_capacity(128);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .unwrap();
        for event in events.iter() {
            match event.token() {
                RECEIVER => {
                    let packet = socket.recv()?;
                    println!("{:?}", packet);
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}
