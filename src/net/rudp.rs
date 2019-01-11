use crate::{
    config::SocketConfig,
    packet::Packet,
    error::NetworkResult
};
use std::{
    self,
    io::{self, Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc
};
use mio;

pub struct RudpSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    receive_buffer: Vec<u8>,
}

impl RudpSocket {
    pub fn bind<A: ToSocketAddrs>(addresses: A, config: SocketConfig) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        let buffer_size = config.receive_buffer_size;
        Ok(Self::new(socket, config,vec![0; buffer_size]))
    }

    fn new(socket: mio::net::UdpSocket, config: SocketConfig, receive_buffer: Vec<u8>) -> Self {
        Self {
            socket,
            config,
            receive_buffer,
        }
    }
}
