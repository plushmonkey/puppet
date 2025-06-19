use crate::clock::*;
use crate::net::connection::{Connection, ConnectionState};
use crate::net::packet::bi::*;
use crate::net::packet::c2s::*;
use crate::net::packet::s2c::*;
use crate::ship::Ship;

pub mod arena_settings;
pub mod clock;
pub mod net;
pub mod player;
pub mod ship;

fn main() -> anyhow::Result<()> {
    let username = "test";
    let password = "none";
    let remote_ip = "127.0.0.1";
    let remote_port = 5000;

    let mut connection = Connection::new(remote_ip, remote_port)?;
    let mut last_position_tick = LocalTick::now();

    let key = 0;
    let encrypt_request = EncryptionRequestMessage::new(key);

    connection.state = ConnectionState::EncryptionHandshake;
    connection.send(&encrypt_request)?;

    loop {
        let now = LocalTick::now();

        let message = connection.tick();
        if let Err(e) = message {
            println!("Error: {}", e);
            if e.is::<std::io::Error>() {
                break;
            }
            continue;
        }

        let message = message.unwrap();

        if let Some(message) = message {
            match message {
                ServerMessage::Core(kind) => match kind {
                    CoreServerMessage::EncryptionResponse(encrypt_response) => {
                        println!("Encryption response: {}", encrypt_response.key);

                        let password = PasswordMessage::new(
                            username, password, false, 0x1231241, 240, 0x86, 123412,
                        );

                        connection.send_reliable(&password)?;

                        let sync_request = SyncRequestMessage::new(2, 2);
                        connection.send(&sync_request)?;
                    }
                    _ => {}
                },
                ServerMessage::Game(kind) => match kind {
                    GameServerMessage::Chat(chat) => {
                        println!("{}", chat.message);
                    }
                    GameServerMessage::PasswordResponse(password_response) => {
                        println!("Got password response: {}", password_response.response);

                        let arena_request = ArenaJoinMessage::new(
                            Ship::Spectator,
                            1920,
                            1080,
                            ArenaRequest::AnyPublic,
                        );

                        connection.send(&arena_request)?;
                    }
                    GameServerMessage::ArenaSettings(_) => {
                        println!("Received arena settings:");
                        // println!("{:?}", settings);
                    }
                    GameServerMessage::PlayerEntering(entering) => {
                        for entry in entering.players {
                            println!("Player {} is entering the arena", entry.name);
                        }
                    }
                    GameServerMessage::MapInformation(info) => {
                        println!("Map name: {}", info.filename);
                        // TODO: Check if we have the map and request if we don't.
                        connection.state = ConnectionState::Playing;

                        let chat = SendChatMessage::public("?arena");

                        connection.send(&chat)?;
                    }
                    GameServerMessage::ArenaDirectory(directory) => {
                        println!("directory: {:?}", directory);
                    }
                    _ => {}
                },
            }
        }

        match connection.state {
            ConnectionState::Playing => {
                if now.diff(&last_position_tick) > 300 {
                    let position = PositionMessage {
                        direction: 0,
                        timestamp: connection.get_server_tick(),
                        x_position: 0,
                        y_position: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        togglables: 0,
                        bounty: 0,
                        energy: 0,
                        weapon_info: 0,
                    };

                    connection.send(&position)?;

                    last_position_tick = now;
                }
            }
            ConnectionState::Disconnected => {
                break;
            }
            _ => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}
