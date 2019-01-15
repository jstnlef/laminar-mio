use crate::{
    config::SocketConfig,
    net::connection::ActiveConnections,
    error::{NetworkError, NetworkErrorKind, NetworkResult},
    packet::Packet,
};
use mio::{Evented, Events, Poll, PollOpt, Ready, Token};
use std::{
    self,
    io::{self, Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
    sync::mpsc,
    time::Duration,
};

const RECEIVER: Token = Token(0);
const SENDER: Token = Token(1);

/// An RUDP socket implementation with configurable reliability and ordering guarantees.
pub struct RudpSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    connections: ActiveConnections,
    receive_buffer: Vec<u8>,
    packet_sender: mpsc::Sender<Packet>,

    // TODO: Have a send buffer for packets. When the caller wants to send a packet, the packet will
    // get placed in this buffer. Then, when the socket is ready to send, send all of the buffered
    // packets at once.
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

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> NetworkResult<()> {
        let poll = Poll::new()?;

        poll.register(self, RECEIVER, Ready::readable(), PollOpt::edge())?;
//        poll.register(self, SENDER, Ready::writable(), PollOpt::edge())?;

        let mut events = Events::with_capacity(self.config.socket_event_buffer_size);
        let events_ref = &mut events;
        loop {
            // TODO: Check for idle connections here.
            poll.poll(events_ref, self.config.socket_polling_timeout)?;
            self.process_events(events_ref)?;
        }
    }

    fn process_events(&mut self, events: &mut Events) -> NetworkResult<()> {
        for event in events.iter() {
            match event.token() {
                RECEIVER => {
                    let packet = self.recv()?;
                    self.packet_sender.send(packet);
                },
//                SENDER => {
//
//                }
                _ => unreachable!(),
            }
        }
        Ok(())
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
                connections: ActiveConnections::new(),
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
