use crate::net::connection::{Connection, ConnectionState};
use crate::net::packet::c2s::*;
use crate::net::packet::s2c::*;
use crate::ship::Ship;

pub mod clock;
pub mod net;
pub mod ship;

fn main() -> anyhow::Result<()> {
    let remote_ip = "127.0.0.1";
    let remote_port = 5000;

    let mut connection = Connection::new(remote_ip, remote_port)?;

    let key = 0;
    let encrypt_request = EncryptionRequestPacket::new(key);

    connection.state = ConnectionState::EncryptionHandshake;
    connection.send(&encrypt_request)?;

    while !matches!(connection.state, ConnectionState::Disconnected) {
        let message = connection.tick();
        if let Err(e) = message {
            println!("Error: {}", e);
            continue;
        }

        let message = message.unwrap();

        if let Some(message) = message {
            match message {
                ServerMessage::Core(kind) => match kind {
                    CoreServerMessage::EncryptionResponse(encrypt_response) => {
                        println!("Encryption response: {}", encrypt_response.key);

                        let password = PasswordPacket::new(
                            "test", "none", false, 0x1231241, 240, 0x86, 123412,
                        );

                        connection.send(&password)?;
                    }
                    CoreServerMessage::Disconnect => {
                        println!("test");
                    }
                    _ => {}
                },
                ServerMessage::Game(kind) => match kind {
                    GameServerMessage::PasswordResponse(password_response) => {
                        println!("Got password response: {}", password_response.response);

                        let arena_request = ArenaJoinPacket::new(
                            Ship::Spectator,
                            1920,
                            1080,
                            ArenaRequest::AnyPublic,
                        );

                        connection.send(&arena_request)?;
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
