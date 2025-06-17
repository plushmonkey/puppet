use crate::net::packet::s2c::*;
use crate::net::packet::sequencer::*;
use crate::net::packet::{Packet, Serialize};

use anyhow::{Result, anyhow};
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    str::FromStr,
};

pub enum ConnectionState {
    EncryptionHandshake,
    Authentication,
    Registering,
    ArenaLogin,
    MapDownload,
    Playing,
    Disconnected,
}

pub struct Connection {
    pub remote_addr: SocketAddr,
    pub socket: UdpSocket,
    pub state: ConnectionState,
    sequencer: PacketSequencer,
}

impl Connection {
    pub fn new(remote_ip: &str, remote_port: u16) -> Result<Self> {
        let remote_addr = std::net::Ipv4Addr::from_str(remote_ip)?;
        let remote_addr = SocketAddr::new(IpAddr::V4(remote_addr), remote_port);
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_nonblocking(true)?;

        Ok(Self {
            remote_addr,
            socket,
            state: ConnectionState::Disconnected,
            sequencer: PacketSequencer::new(),
        })
    }

    pub fn send<T>(&self, message: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.send_packet(&message.serialize())
    }

    pub fn send_packet(&self, packet: &Packet) -> Result<()> {
        if packet.size == 0 {
            return Err(anyhow!("packet must not be empty"));
        }

        let buf = packet.data();

        println!("Sending {:?}", buf);

        self.socket.send_to(buf, &self.remote_addr)?;

        Ok(())
    }

    pub fn tick(&mut self) -> Result<Option<ServerMessage>> {
        let packet = self.recv_packet()?;

        // If we received a packet and it got processed into a complete message, return it.
        if let Some(packet) = packet {
            let result = self.process_packet(&packet);

            if let Ok(Some(_)) = result {
                return result;
            }
        }

        // Grab the next reliable message off of the queue if possible.
        if let Some(rel) = self.sequencer.pop_process_queue() {
            let packet = Packet::new(&rel.message[..rel.size]);
            return self.process_packet(&packet);
        }

        Ok(None)
    }

    fn process_packet(&mut self, packet: &Packet) -> Result<Option<ServerMessage>> {
        let message = ServerMessage::parse(packet)?;

        if let Some(message) = &message {
            match message {
                ServerMessage::Core(kind) => match kind {
                    CoreServerMessage::EncryptionResponse(_) => {
                        //
                    }
                    CoreServerMessage::ReliableAck(ack) => {
                        self.sequencer.handle_ack(ack.id);
                        println!("Got reliable ack {}", ack.id);
                    }
                    CoreServerMessage::ReliableData(rel) => {
                        self.sequencer.handle_reliable_message(rel.id, &rel.data);
                        println!("Got reliable data {:?}", &rel.data.data[..rel.data.size]);
                        let ack = Packet::empty()
                            .concat_u8(0x00)
                            .concat_u8(0x04)
                            .concat_u32(rel.id);
                        if let Err(e) = self.send_packet(&ack) {
                            println!("Error: {}", e);
                        }
                    }
                    CoreServerMessage::Disconnect => {
                        println!("Got disconnect order.");
                        self.state = ConnectionState::Disconnected;
                    }
                },
                _ => {}
            }
        }

        return Ok(message);
    }

    fn recv_packet(&self) -> Result<Option<Packet>> {
        let mut packet = Packet::empty();

        let (size, _) = match self.socket.recv_from(&mut packet.data[..]) {
            Ok(r) => r,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(None);
                }

                return Err(anyhow!(e));
            }
        };

        packet.size = size;

        Ok(Some(packet))
    }
}
