use laminar::{
    config::SocketConfig,
    error::NetworkError,
    net::{RudpSocket, SocketEvent},
};
use std::{net::SocketAddr, sync::mpsc, thread};

fn main() -> Result<(), Box<NetworkError>> {
    let config = SocketConfig::default();
    let local_address: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let (mut socket, packet_sender, event_receiver) =
        RudpSocket::bind(local_address, config).unwrap();

    let _thread = thread::spawn(move || socket.start_polling());

    loop {
        let event = event_receiver.recv().unwrap();
        match event {
            SocketEvent::Packet(packet) => println!("{:?}", packet),
            SocketEvent::TimeOut(address) => println!("Client {:?} timed out", address),
            _ => {}
        }
    }
}
