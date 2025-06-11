use net::connection::{Connection, ConnectionState};
use net::packet::Packet;

pub mod clock;
pub mod net;

fn main() -> anyhow::Result<()> {
    let remote_ip = "127.0.0.1";
    let remote_port = 5000;

    let mut connection = Connection::new(remote_ip, remote_port)?;

    let key = 0;
    let version = 0x01;
    let encrypt_request = Packet::empty()
        .concat_u8(0x00)
        .concat_u8(0x01)
        .concat_u32(key)
        .concat_u16(version);

    connection.state = ConnectionState::EncryptionHandshake;
    connection.send(&encrypt_request)?;

    while !matches!(connection.state, ConnectionState::Disconnected) {
        let packet = connection.recv()?;

        if let Some(packet) = packet {
            println!("recv: {:?}", packet.data());
        }
    }

    Ok(())
}
