use crate::{
    config::SocketConfig,
    errors::LaminarError,
    net::{connection::ActiveConnections, events::SocketEvent},
    packet::Packet,
};
use mio::{Evented, Events, Poll, PollOpt, Ready, Token};
use std::{
    self, io, mem,
    net::{SocketAddr, ToSocketAddrs},
    sync::mpsc,
};
use log::error;

const SOCKET: Token = Token(0);

/// A reliable UDP socket implementation with configurable reliability and ordering guarantees.
pub struct LaminarSocket {
    socket: mio::net::UdpSocket,
    config: SocketConfig,
    connections: ActiveConnections,
    receive_buffer: Vec<u8>,
    event_sender: mpsc::Sender<SocketEvent>,
    packet_receiver: mpsc::Receiver<Packet>,
}

impl LaminarSocket {
    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    pub fn bind<A: ToSocketAddrs>(
        addresses: A,
        config: SocketConfig,
    ) -> io::Result<(Self, mpsc::Sender<Packet>, mpsc::Receiver<SocketEvent>)> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        Ok(Self::new(socket, config))
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> io::Result<()> {
        let poll = Poll::new()?;

        poll.register(self, SOCKET, Ready::readable(), PollOpt::edge())?;

        let mut events = Events::with_capacity(self.config.socket_event_buffer_size());
        let events_ref = &mut events;
        // Packet receiver MUST only be used in this method.
        let packet_receiver = mem::replace(&mut self.packet_receiver, mpsc::channel().1);
        // Nothing should break out of this loop!
        loop {
            self.handle_idle_clients();
            if let Err(e) = poll.poll(events_ref, self.config.socket_polling_timeout()) {
                error!("Error polling the socket: {:?}", e);
            }
            if let Err(e) = self.process_events(events_ref) {
                error!("Error processing events: {:?}", e);
            }
            // XXX: I'm fairly certain this isn't exactly safe. I'll likely need to add some
            // handling for when the socket is blocked on send. Worth some more research.
            // Alternatively, I'm sure the Tokio single threaded runtime does handle this for us
            // so maybe it's work switching to that while providing the same interface?
            for packet in packet_receiver.try_iter() {
                if let Err(e) = self.send_to(packet) {
                    error!("Error sending packet: {:?}", e);
                }
            }
        }
    }

    /// Iterate through all of the idle connections based on `idle_connection_timeout` config and
    /// remove them from the active connections. For each connection removed, we will send a
    /// `SocketEvent::TimeOut` event to the `event_sender` channel.
    fn handle_idle_clients(&mut self) {
        let idle_addresses = self
            .connections
            .idle_connections(self.config.idle_connection_timeout());

        for address in idle_addresses {
            self.connections.remove_connection(&address);
            self.event_sender.send(SocketEvent::TimeOut(address));
        }
    }

    /// Process events received from the mio socket.
    fn process_events(&mut self, events: &mut Events) -> io::Result<()> {
        for event in events.iter() {
            match event.token() {
                SOCKET => {
                    if event.readiness().is_readable() {
                        loop {
                            match self.receive_from() {
                                Ok(Some(packet)) => {
                                    self.event_sender.send(SocketEvent::Packet(packet));
                                }
                                Ok(None) => continue,
                                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => error!("{:?}", e)
                            };
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    /// Returns the socket address that this socket was created from.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Serializes and sends a `Packet` on the socket. On success, returns the number of bytes written.
    fn send_to(&mut self, packet: Packet) -> io::Result<usize> {
        let connection = self
            .connections
            .get_or_insert_connection(&packet.address(), &self.config);
        let mut processed = connection.process_outgoing(packet)?;
        let mut bytes_written = 0;

        // TODO: Is this where we want to send dropped packets?
        if connection.has_dropped_packets() {
            for payload in connection.drain_dropped_packets() {
                bytes_written += self.socket.send_to(&payload, &processed.address())?;
            }
        }

        let address = processed.address();
        for fragment in processed.fragments(
            self.config.fragment_size_bytes(),
            self.config.max_fragments(),
        )? {
            bytes_written += self.socket.send_to(fragment, &address)?;
        }

        Ok(bytes_written)
    }

    /// Receives a single message from the socket. On success, returns the packet containing origin and data.
    fn receive_from(&mut self) -> io::Result<Option<Packet>> {
        let (recv_len, address) = self.socket.recv_from(&mut self.receive_buffer)?;
        if recv_len <= 0 {
            return Err(LaminarError::ReceivedDataTooShort.into());
        }

        let received_payload = &self.receive_buffer[..recv_len];
        let connection = self
            .connections
            .get_or_insert_connection(&address, &self.config);
        connection.process_incoming(received_payload)
    }

    fn new(
        socket: mio::net::UdpSocket,
        config: SocketConfig,
    ) -> (Self, mpsc::Sender<Packet>, mpsc::Receiver<SocketEvent>) {
        let (event_sender, event_receiver) = mpsc::channel();
        let (packet_sender, packet_receiver) = mpsc::channel();
        let buffer_size = config.receive_buffer_size_bytes();
        (
            Self {
                socket,
                config,
                connections: ActiveConnections::new(),
                receive_buffer: vec![0; buffer_size],
                event_sender,
                packet_receiver,
            },
            packet_sender,
            event_receiver,
        )
    }
}

impl Evented for LaminarSocket {
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
