use crate::clock::*;
use crate::map::Map;
use crate::net::connection::{Connection, ConnectionState};
use crate::net::packet::bi::*;
use crate::net::packet::c2s::*;
use crate::net::packet::s2c::*;
use crate::ship::Ship;
use ctrlc;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::fs::{self, DirBuilder};
use std::sync::mpsc::channel;

pub mod arena_settings;
pub mod checksum;
pub mod clock;
pub mod map;
pub mod net;
pub mod player;
pub mod ship;

fn build_zone_directory(zone: &str) -> anyhow::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .create(format!("zones/{}", zone))?;
    Ok(())
}

fn get_zone_path(zone: &str, filename: &str) -> String {
    format!("zones/{}/{}", zone, filename)
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel();

    let _ = ctrlc::set_handler(move || {
        let _ = tx.send(());
    });

    let username = "test";
    let password = "none";
    let zone = "local";
    let remote_ip = "127.0.0.1";
    let remote_port = 5000;

    let mut connection = Connection::new(remote_ip, remote_port)?;
    let mut last_position_tick = LocalTick::now();

    let key = 0;
    let encrypt_request = EncryptionRequestMessage::new(key);

    let mut map = Map::empty(0, "");

    connection.state = ConnectionState::EncryptionHandshake;
    connection.send(&encrypt_request)?;

    loop {
        // Exit loop if we receive a control-c signal.
        if let Ok(_) = rx.try_recv() {
            break;
        }

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
                        if !chat.message.is_empty() {
                            println!("{}", chat.message);
                        }
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
                        //println!("Received arena settings:");
                        // println!("{:?}", settings);
                    }
                    GameServerMessage::PlayerEntering(entering) => {
                        for entry in entering.players {
                            println!("{} entered arena", entry.name);
                        }
                    }
                    GameServerMessage::MapInformation(info) => {
                        println!("Map name: {}", info.filename);

                        connection.state = ConnectionState::MapDownload;

                        let chat = SendChatMessage::public("?arena");
                        connection.send(&chat)?;

                        let map_path = get_zone_path(zone, &info.filename);
                        let map_data = fs::read(map_path);

                        if let Ok(map_data) = map_data {
                            let checksum = checksum::crc32(&map_data);

                            if checksum == info.checksum {
                                map = Map::new(info.checksum, &info.filename, &map_data);
                                connection.state = ConnectionState::Playing;
                            }
                        } else if let Err(e) = map_data {
                            println!("Map read error: {}", e);
                        }

                        if matches!(connection.state, ConnectionState::MapDownload) {
                            // Request
                            let map_request = MapRequestMessage {};
                            connection.send(&map_request)?;

                            connection.state = ConnectionState::MapDownload;

                            map = Map::empty(info.checksum, &info.filename);
                        }
                    }
                    GameServerMessage::CompressedMap(compressed) => {
                        if compressed.filename == map.filename {
                            let inflated = decompress_to_vec_zlib(compressed.data.as_slice());

                            match inflated {
                                Ok(inflated) => {
                                    let map_path = get_zone_path(zone, &compressed.filename);

                                    if let Err(e) = build_zone_directory(zone) {
                                        println!("Error creating zone directory: {}", e);
                                    }

                                    if let Err(e) = fs::write(map_path, inflated.as_slice()) {
                                        println!("Error writing map: {}", e);
                                    }

                                    map.data = inflated;
                                }
                                Err(e) => {
                                    println!("Error: {}", e);
                                    break;
                                }
                            }
                        }
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

    // Always send disconnect when we are exiting so we don't linger on the server.
    let disconnect = DisconnectMessage {};
    connection.send(&disconnect)?;

    Ok(())
}
