use crate::clock::*;
use crate::net::packet::bi::HugeChunkCancelAckMessage;
use crate::net::packet::bi::ReliableDataMessage;
use crate::net::packet::s2c::*;
use crate::net::packet::sequencer::*;
use crate::net::packet::{MAX_PACKET_SIZE, Packet, Serialize};
use crate::player::PlayerId;

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
    pub tick_diff: i32,
    pub player_id: PlayerId,
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
            tick_diff: 0,
            player_id: PlayerId::invalid(),
        })
    }

    pub fn get_server_tick(&self) -> ServerTick {
        ServerTick::now(self.tick_diff)
    }

    pub fn send<T>(&self, message: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.send_packet(&message.serialize())
    }

    pub fn send_reliable<T>(&mut self, message: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.send_reliable_packet(&message.serialize())
    }

    pub fn send_packet(&self, packet: &Packet) -> Result<()> {
        if packet.size == 0 {
            return Err(anyhow!("packet must not be empty"));
        }

        let buf = packet.data();

        println!("Sending {:02x?}", buf);

        self.socket.send_to(buf, &self.remote_addr)?;

        Ok(())
    }

    pub fn send_reliable_packet(&mut self, packet: &Packet) -> Result<()> {
        if packet.size == 0 {
            return Err(anyhow!("reliable packet payload must not be empty"));
        }
        if packet.size + 6 > MAX_PACKET_SIZE {
            return Err(anyhow!("reliable packet payload too large"));
        }

        let id = self.sequencer.next_reliable_gen_id;
        self.sequencer.increment_id();

        let reliable = ReliableDataMessage {
            id,
            data: packet.clone(),
        };

        let packet = reliable.serialize();
        let buf = packet.data();

        self.sequencer.push_reliable_sent(id, buf);

        println!("Sending {:02x?}", buf);

        self.socket.send_to(buf, &self.remote_addr)?;

        Ok(())
    }

    pub fn tick(&mut self) -> Result<Option<ServerMessage>> {
        if let Some(message) = self.sequencer.tick() {
            self.send_packet(&message)?;
        }

        let packet = self.recv_packet()?;

        // If we received a packet and it got processed into a complete message, return it.
        if let Some(packet) = packet {
            let result = ServerMessage::parse(&packet.data[..packet.size])?;
            if let Some(message) = &result {
                self.process_packet(&message);
            }

            return Ok(result);
        }

        // Grab the next reliable message / cluster message off of the queue if possible.
        let sequence_message = self.sequencer.pop_process_queue()?;

        if let Some(message) = &sequence_message {
            self.process_packet(&message);
            return Ok(sequence_message);
        }

        Ok(None)
    }

    fn process_packet(&mut self, message: &ServerMessage) {
        match message {
            ServerMessage::Core(kind) => match kind {
                CoreServerMessage::EncryptionResponse(_) => {
                    //
                }
                CoreServerMessage::ReliableAck(ack) => {
                    self.sequencer.handle_ack(ack.id);
                    // println!("Got reliable ack {}", ack.id);
                }
                CoreServerMessage::ReliableData(rel) => {
                    self.sequencer.handle_reliable_message(rel.id, &rel.data);
                    // println!("Got reliable data {:?}", &rel.data.data[..rel.data.size]);
                    let ack = Packet::empty()
                        .concat_u8(0x00)
                        .concat_u8(0x04)
                        .concat_u32(rel.id);
                    if let Err(e) = self.send_packet(&ack) {
                        println!("Error: {}", e);
                    }
                }
                CoreServerMessage::SyncResponse(sync) => {
                    let server_timestamp = sync.server_timestamp.value() as i32;
                    let current_timestamp = LocalTick::now().value() as i32;
                    let rtt = current_timestamp - sync.request_timestamp.value() as i32;

                    let current_time_diff = ((rtt * 3) / 5) + server_timestamp - current_timestamp;

                    self.tick_diff = current_time_diff;
                }
                CoreServerMessage::Disconnect => {
                    println!("Got disconnect order.");
                    self.state = ConnectionState::Disconnected;
                }
                CoreServerMessage::SmallChunkBody(chunk) => {
                    self.sequencer.handle_small_chunk_body(&chunk.data);
                }
                CoreServerMessage::SmallChunkTail(tail) => {
                    self.sequencer.handle_small_chunk_tail(&tail.data);
                }
                CoreServerMessage::HugeChunk(chunk) => {
                    self.sequencer.handle_huge_chunk(chunk);
                }
                CoreServerMessage::HugeChunkCancel => {
                    self.sequencer.handle_huge_chunk_cancel();

                    let cancel = HugeChunkCancelAckMessage {};
                    if let Err(e) = self.send(&cancel) {
                        println!("Error: {}", e);
                    }
                }
                CoreServerMessage::HugeChunkCancelAck => {
                    //
                }
                CoreServerMessage::Cluster(cluster) => {
                    self.sequencer.handle_cluster(cluster);
                }
            },
            ServerMessage::Game(kind) => match kind {
                GameServerMessage::PlayerId(message) => {
                    self.player_id = message.id;
                }
                _ => {}
            },
        }
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

        println!("Recv: {:02x?}", &packet.data[..size]);

        Ok(Some(packet))
    }
}
