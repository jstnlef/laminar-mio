use crate::{
    config::SocketConfig,
    error::{NetworkError, NetworkErrorKind, NetworkResult},
    net::{connection::ActiveConnections, events::SocketEvent},
    packet::Packet,
};
use mio::{Evented, Events, Poll, PollOpt, Ready, Token};
use std::{
    self,
    io::{self},
    net::{SocketAddr, ToSocketAddrs},
    sync::mpsc
};

const RECEIVER: Token = Token(0);
const SENDER: Token = Token(1);

/// An RUDP socket implementation with configurable reliability and ordering guarantees.
pub struct RudpSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    connections: ActiveConnections,
    receive_buffer: Vec<u8>,
    event_sender: mpsc::Sender<SocketEvent>,
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
    ) -> NetworkResult<(Self, mpsc::Receiver<SocketEvent>)> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        Ok(Self::new(socket, config))
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> NetworkResult<()> {
        let poll = Poll::new()?;

        poll.register(self, RECEIVER, Ready::readable(), PollOpt::edge())?;

        let mut events = Events::with_capacity(self.config.socket_event_buffer_size);
        let events_ref = &mut events;
        loop {
            self.handle_idle_clients();
            poll.poll(events_ref, self.config.socket_polling_timeout)?;
            self.process_events(events_ref)?;
        }
    }

    fn handle_idle_clients(&mut self) {
        let idle_addresses = self
            .connections
            .idle_connections(self.config.idle_connection_timeout);

        for address in idle_addresses {
            self.connections.remove_connection(&address);
            self.event_sender.send(SocketEvent::TimeOut(address));
        }
    }

    fn process_events(&mut self, events: &mut Events) -> NetworkResult<()> {
        for event in events.iter() {
            match event.token() {
                RECEIVER => {
                    let packet = self.recv()?;
                    self.event_sender.send(SocketEvent::Packet(packet));
                }
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
            let mut connection = self.connections.get_or_insert_connection(&address);
            connection.packet_received();
            // XXX: Does an allocation to copy the bytes into packet. Maybe it shouldn't?
            Ok(Packet::unreliable(address, payload.to_owned()))
        } else {
            Err(NetworkError::new(NetworkErrorKind::ReceivedDataToShort))
        }
    }

    fn new(
        socket: mio::net::UdpSocket,
        config: SocketConfig,
    ) -> (Self, mpsc::Receiver<SocketEvent>) {
        let (event_sender, event_receiver) = mpsc::channel();
        let buffer_size = config.receive_buffer_size;
        (
            Self {
                socket,
                config,
                connections: ActiveConnections::new(),
                receive_buffer: vec![0; buffer_size],
                event_sender,
            },
            event_receiver,
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
