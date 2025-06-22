use crate::arena_settings::ArenaSettings;
use crate::checksum;
use crate::clock::*;
use crate::map::Map;
use crate::math::{Position, Velocity};
use crate::net::connection::{Connection, ConnectionState};
use crate::net::packet::bi::*;
use crate::net::packet::c2s::*;
use crate::net::packet::s2c::*;
use crate::player::*;
use crate::ship::Ship;

use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::fs::{self, DirBuilder};

fn build_zone_directory(zone: &str) -> anyhow::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .create(format!("zones/{}", zone))?;
    Ok(())
}

fn get_zone_path(zone: &str, filename: &str) -> String {
    format!("zones/{}/{}", zone, filename)
}

pub struct Client {
    pub connection: Connection,
    pub map: Map,
    pub settings: Option<Box<ArenaSettings>>,
    pub last_position_tick: LocalTick,
    pub player_manager: PlayerManager,

    pub username: String,
    pub password: String,
    pub zone: String,
}

impl Client {
    pub fn new(
        username: &str,
        password: &str,
        zone: &str,
        remote_ip: &str,
        remote_port: u16,
    ) -> anyhow::Result<Client> {
        let mut connection = Connection::new(remote_ip, remote_port)?;

        let key = 0;
        let encrypt_request = EncryptionRequestMessage::new(key);

        connection.state = ConnectionState::EncryptionHandshake;
        connection.send(&encrypt_request)?;

        Ok(Client {
            connection,
            map: Map::empty(0, ""),
            settings: None,
            last_position_tick: LocalTick::now(),
            player_manager: PlayerManager::new(),
            username: username.to_owned(),
            password: password.to_owned(),
            zone: zone.to_owned(),
        })
    }

    pub fn run(&mut self, rx: std::sync::mpsc::Receiver<()>) -> anyhow::Result<()> {
        loop {
            // Exit loop if we receive a control-c signal.
            if let Ok(_) = rx.try_recv() {
                break;
            }

            let now = LocalTick::now();

            loop {
                let message = self.connection.tick();
                if let Err(e) = message {
                    println!("Error: {}", e);
                    if e.is::<std::io::Error>() {
                        break;
                    }
                    continue;
                }

                let message = message.unwrap();

                if let Some(message) = message {
                    self.process_message(message)?;
                } else {
                    // We are done processing everything now.
                    break;
                }
            }

            match self.connection.state {
                ConnectionState::Playing => {
                    if now.diff(&self.last_position_tick) > 300 {
                        let position = PositionMessage {
                            direction: 0,
                            timestamp: self.connection.get_server_tick(),
                            x_position: 0,
                            y_position: 0,
                            x_velocity: 0,
                            y_velocity: 0,
                            togglables: 0,
                            bounty: 0,
                            energy: 0,
                            weapon_info: 0,
                        };

                        self.connection.send(&position)?;

                        self.last_position_tick = now;
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
        self.connection.send(&disconnect)?;

        Ok(())
    }

    fn process_core_message(&mut self, message: &CoreServerMessage) -> anyhow::Result<()> {
        match message {
            CoreServerMessage::EncryptionResponse(encrypt_response) => {
                println!("Encryption response: {}", encrypt_response.key);

                let password = PasswordMessage::new(
                    &self.username,
                    &self.password,
                    false,
                    0x1231241,
                    240,
                    0x86,
                    123412,
                );

                self.connection.send_reliable(&password)?;

                let sync_request = SyncRequestMessage::new(2, 2);
                self.connection.send(&sync_request)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn process_game_message(&mut self, message: &GameServerMessage) -> anyhow::Result<()> {
        match message {
            GameServerMessage::Chat(chat) => {
                if !chat.message.is_empty() {
                    println!("{}", chat.message);
                }
            }
            GameServerMessage::PasswordResponse(password_response) => {
                println!("Got password response: {}", password_response.response);

                let arena_request =
                    ArenaJoinMessage::new(Ship::Spectator, 1920, 1080, ArenaRequest::AnyPublic);

                self.connection.send(&arena_request)?;
            }
            GameServerMessage::ArenaSettings(settings_message) => {
                println!("Received arena settings");
                // println!("{:?}", settings);
                self.settings = Some(settings_message.clone());
            }
            GameServerMessage::SynchronizationRequest(sync) => {
                if sync.checksum_key != 0 && self.map.checksum != 0 {
                    // Send security packet
                    println!("Sync requested");

                    if let Some(settings) = &self.settings {
                        let settings_checksum =
                            checksum::settings_checksum(sync.checksum_key, &settings.raw_bytes)?;
                        let exe_checksum = checksum::vie_checksum(sync.checksum_key);
                        let level_checksum = checksum::checksum_map(&self.map, sync.checksum_key);

                        let response = SecurityMessage::new(
                            0,
                            settings_checksum,
                            exe_checksum,
                            level_checksum,
                        );
                        println!("Sending security packet");
                        self.connection.send(&response)?;
                    }
                }
            }
            GameServerMessage::PlayerEntering(entering) => {
                for entry in &entering.players {
                    let mut player = Player::new(entry.player_id, &entry.name, &entry.squad);

                    player.flag_count = entry.flag_count;
                    player.attach_parent = entry.attach_parent;

                    // If there was someone already in this place, say that they left.
                    // This can happen when joining at the same exact time as other players.
                    if let Some(old_player) = self.player_manager.add_player(player) {
                        println!("{} left arena", old_player.name);
                    }

                    println!("{} entered arena", entry.name);
                }
            }
            GameServerMessage::PlayerLeaving(leaving) => {
                if let Some(player) = self.player_manager.remove_player(&leaving.player_id) {
                    println!("{} left arena", player.name);
                }
            }
            GameServerMessage::SmallPosition(message) => {
                if let Some(player) = self.player_manager.get_mut(&message.player_id) {
                    if player.last_position_timestamp < message.timestamp {
                        player.position = Position::new(message.x as u32, message.y as u32);
                        player.velocity =
                            Velocity::new(message.x_velocity as i32, message.y_velocity as i32);
                        player.direction = message.direction;
                        player.bounty = message.bounty as u16;
                        player.status = message.status;
                        player.ping = message.ping;
                        player.last_position_timestamp = message.timestamp;

                        println!("{} at {:?}", player.name, player.position);
                    }
                }
            }
            GameServerMessage::LargePosition(message) => {
                if let Some(player) = self.player_manager.get_mut(&message.player_id) {
                    if player.last_position_timestamp < message.timestamp {
                        player.position = Position::new(message.x as u32, message.y as u32);
                        player.velocity =
                            Velocity::new(message.x_velocity as i32, message.y_velocity as i32);
                        player.direction = message.direction;
                        player.bounty = message.bounty;
                        player.status = message.status;
                        player.ping = message.ping;
                        player.last_position_timestamp = message.timestamp;

                        println!("{} at {:?}", player.name, player.position);
                    }
                }
            }
            GameServerMessage::MapInformation(info) => {
                println!("Map name: {}", info.filename);

                self.connection.state = ConnectionState::MapDownload;

                let chat = SendChatMessage::public("?arena");
                self.connection.send(&chat)?;

                let map_path = get_zone_path(&self.zone, &info.filename);
                let map_data = fs::read(map_path);

                if let Ok(map_data) = map_data {
                    let checksum = checksum::crc32(&map_data);

                    if checksum == info.checksum {
                        if let Some(new_map) = Map::new(info.checksum, &info.filename, &map_data) {
                            self.map = new_map;
                        } else {
                            println!("Map read errorr: failed to load tiles");
                        }
                        self.connection.state = ConnectionState::Playing;
                    }
                } else if let Err(e) = map_data {
                    println!("Map read error: {}", e);
                }

                if matches!(self.connection.state, ConnectionState::MapDownload) {
                    // Request
                    let map_request = MapRequestMessage {};
                    self.connection.send(&map_request)?;

                    self.connection.state = ConnectionState::MapDownload;

                    self.map = Map::empty(info.checksum, &info.filename);
                }
            }
            GameServerMessage::CompressedMap(compressed) => {
                if compressed.filename == self.map.filename {
                    let inflated = decompress_to_vec_zlib(compressed.data.as_slice());

                    match inflated {
                        Ok(inflated) => {
                            let map_path = get_zone_path(&self.zone, &compressed.filename);

                            if let Err(e) = build_zone_directory(&self.zone) {
                                println!("Error creating zone directory: {}", e);
                            }

                            if let Err(e) = fs::write(map_path, inflated.as_slice()) {
                                println!("Error writing map: {}", e);
                            }

                            if let Some(new_map) =
                                Map::new(self.map.checksum, &self.map.filename, &inflated)
                            {
                                self.map = new_map;
                            } else {
                                println!("Map read error: failed to load tiles");
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
            GameServerMessage::ArenaDirectory(directory) => {
                println!("directory: {:?}", directory);
            }
            _ => {}
        }

        Ok(())
    }

    fn process_message(&mut self, message: ServerMessage) -> anyhow::Result<()> {
        match message {
            ServerMessage::Core(core_message) => self.process_core_message(&core_message),
            ServerMessage::Game(game_message) => self.process_game_message(&game_message),
        }
    }
}
