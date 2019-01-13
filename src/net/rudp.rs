use crate::{
    config::SocketConfig,
    error::{NetworkError, NetworkErrorKind, NetworkResult},
    packet::Packet,
};
use mio::Token;
use std::{
    self,
    io::{self, Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
};

/// An RUDP socket implementation with configurable reliability and ordering guarantees.
pub struct RudpSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    //    connections: ActiveConnections,
    receive_buffer: Vec<u8>,
}

impl RudpSocket {
    ///
    ///
    pub fn bind<A: ToSocketAddrs>(addresses: A, config: SocketConfig) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        let buffer_size = config.receive_buffer_size;
        Ok(Self::new(socket, config, vec![0; buffer_size]))
    }

    ///
    ///
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    ///
    ///
    pub fn try_clone(&self) -> io::Result<Self> {
        self.socket.try_clone().map(|s| Self {
            socket: s,
            config: self.config.clone(),
            receive_buffer: vec![0; self.config.receive_buffer_size],
        })
    }

    ///
    ///
    pub fn send(&self, packet: Packet) -> io::Result<usize> {
        self.socket.send_to(packet.payload(), &packet.address())
    }

    ///
    ///
    pub fn recv(&mut self) -> NetworkResult<Packet> {
        let (recv_len, address) = self.socket.recv_from(&mut self.receive_buffer)?;
        if recv_len > 0 {
            let payload = &self.receive_buffer[..recv_len];
            // XXX: Does an allocation to copy the bytes into packet. Maybe it shouldn't?
            Ok(Packet::unreliable(address, payload.to_owned()))
        } else {
            Err(NetworkError::new(NetworkErrorKind::ReceivedDataToShort))
        }
    }

    fn new(socket: mio::net::UdpSocket, config: SocketConfig, receive_buffer: Vec<u8>) -> Self {
        Self {
            socket,
            config,
            receive_buffer,
        }
    }
}
