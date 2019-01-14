use crate::{
    config::SocketConfig,
    error::{NetworkError, NetworkErrorKind, NetworkResult},
    packet::Packet,
};
use mio::{Evented, Poll, PollOpt, Ready, Token, Events};
use std::{
    self,
    io::{self, Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
    sync::mpsc,
    time::Duration
};

const RECEIVER: Token = Token(0);

/// An RUDP socket implementation with configurable reliability and ordering guarantees.
pub struct RudpSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    //    connections: ActiveConnections,
    receive_buffer: Vec<u8>,
    packet_sender: mpsc::Sender<Packet>,
}

impl RudpSocket {
    ///
    ///
    pub fn bind<A: ToSocketAddrs>(
        addresses: A,
        config: SocketConfig,
    ) -> NetworkResult<(Self, mpsc::Receiver<Packet>)> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        let buffer_size = config.receive_buffer_size;
        Ok(Self::new(socket, config, vec![0; buffer_size]))
    }

    pub fn run(&mut self) {
        let poll = Poll::new().unwrap();

        poll.register(self, RECEIVER, Ready::readable(), PollOpt::edge())
            .unwrap();

        let mut events = Events::with_capacity(128);

        loop {
            // TODO: Check for idle connections here.
            poll.poll(&mut events, Some(Duration::from_millis(100)))
                .unwrap();
            for event in events.iter() {
                match event.token() {
                    RECEIVER => {
                        let packet = self.recv().unwrap();
                        self.packet_sender.send(packet);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    ///
    ///
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
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

    fn new(
        socket: mio::net::UdpSocket,
        config: SocketConfig,
        receive_buffer: Vec<u8>,
    ) -> (Self, mpsc::Receiver<Packet>) {
        let (packet_sender, packet_receiver) = mpsc::channel();
        (
            Self {
                socket,
                config,
                receive_buffer,
                packet_sender,
            },
            packet_receiver,
        )
    }
}

impl Evented for RudpSocket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.socket.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.socket.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.socket.deregister(poll)
    }
}
