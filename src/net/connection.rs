use crate::net::packet::Packet;
use anyhow::{Result, anyhow};
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    str::FromStr,
};

pub enum ConnectionState {
    EncryptionHandshake,
    Playing,
    Disconnected,
}

pub struct Connection {
    pub remote_addr: SocketAddr,
    pub socket: UdpSocket,
    pub state: ConnectionState,
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
        })
    }

    pub fn send(&self, packet: &Packet) -> Result<()> {
        if packet.size == 0 {
            return Err(anyhow!("packet must not be empty"));
        }

        let buf = packet.data();

        println!("Sending {:?}", buf);

        self.socket.send_to(buf, &self.remote_addr)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Option<Packet>> {
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
