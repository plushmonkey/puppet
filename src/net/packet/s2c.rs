use crate::net::packet::Packet;
use anyhow::{Result, anyhow};
use std::fmt::{self, Debug};

pub enum ServerMessage {
    Core(CoreServerMessage),
    Game(GameServerMessage),
}

pub enum CoreServerMessage {
    EncryptionResponse(EncryptionResponseMessage),
    ReliableData(ReliableDataMessage),
    ReliableAck(ReliableAckMessage),
    Disconnect,
}

pub enum GameServerMessage {
    PlayerId(PlayerIdMessage),
    PasswordResponse(PasswordResponseMessage),
}

// Core messages

pub struct EncryptionResponseMessage {
    pub key: u32,
}

pub struct ReliableDataMessage {
    pub id: u32,
    pub data: Packet,
}

pub struct ReliableAckMessage {
    pub id: u32,
}

// Game messages

pub struct PlayerIdMessage {
    pub id: u16,
}

#[derive(Debug)]
pub enum LoginResponse {
    Ok,
    Unregistered,
    BadPassword,
    ArenaFull,
    LockedOut,
    PermissionOnly,
    SpectateOnly,
    HighPoints,
    ConnectionSlow,
    ServerFull,
    InvalidName,
    OffensiveName,
    NoBiller,
    ServerBusy,
    UsageLow,
    Restricted,
    Demo,
    TooManyDemo,
    DemoDisabled,
}

impl std::fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct PasswordResponseMessage {
    pub response: LoginResponse,
    pub server_version: u32,
    pub registration_request: bool,
    pub news_checksum: u32,
}

impl ServerMessage {
    pub fn parse(packet: &Packet) -> Result<Option<ServerMessage>> {
        if packet.size <= 0 {
            return Err(anyhow!("invalid packet size (0)"));
        }

        let kind = packet.data[0];

        if kind == 0 {
            return ServerMessage::parse_core_packet(&packet);
        }

        return ServerMessage::parse_game_packet(&packet);
    }

    pub fn parse_core_packet(packet: &Packet) -> Result<Option<ServerMessage>> {
        if packet.size < 2 {
            return Err(anyhow!("expected packet type field in core packet"));
        }

        let kind = packet.data[1];

        match kind {
            0x02 => {
                // EncryptionResponse
                if packet.size < 6 {
                    return Err(anyhow!("encryption response was too small"));
                }

                let key = u32::from_le_bytes(packet.data[2..6].try_into().unwrap());

                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::EncryptionResponse(EncryptionResponseMessage { key }),
                )));
            }
            0x03 => {
                // ReliableMessage
                if packet.size < 7 {
                    return Err(anyhow!("reliable message was too small"));
                }

                let id = u32::from_le_bytes(packet.data[2..6].try_into().unwrap());
                let data = Packet::new(&packet.data[6..packet.size]);

                return Ok(Some(ServerMessage::Core(CoreServerMessage::ReliableData(
                    ReliableDataMessage { id, data },
                ))));
            }
            0x04 => {
                // ReliableAck
                if packet.size < 6 {
                    return Err(anyhow!("reliable ack was too small"));
                }

                let id = u32::from_le_bytes(packet.data[2..6].try_into().unwrap());

                return Ok(Some(ServerMessage::Core(CoreServerMessage::ReliableAck(
                    ReliableAckMessage { id },
                ))));
            }
            0x07 => {
                return Ok(Some(ServerMessage::Core(CoreServerMessage::Disconnect)));
            }
            _ => {
                return Err(anyhow!(format!(
                    "invalid core packet type {} received",
                    kind
                )));
            }
        }
    }

    pub fn parse_game_packet(packet: &Packet) -> Result<Option<ServerMessage>> {
        let kind = packet.data[0];

        match kind {
            0x01 => {
                if packet.size < 3 {
                    return Err(anyhow!("player id message was too small"));
                }

                let id = u16::from_le_bytes(packet.data[1..3].try_into().unwrap());

                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerId(
                    PlayerIdMessage { id },
                ))));
            }
            0x0A => {
                if packet.size < 28 {
                    return Err(anyhow!("password packet message was too small"));
                }

                let response_type = packet.data[1];
                let server_version = u32::from_le_bytes(packet.data[2..6].try_into().unwrap());
                let registration_request = packet.data[19];
                let news_checksum = u32::from_le_bytes(packet.data[24..28].try_into().unwrap());

                let response = match response_type {
                    0x00 => LoginResponse::Ok,
                    0x01 => LoginResponse::Unregistered,
                    0x02 => LoginResponse::BadPassword,
                    0x03 => LoginResponse::ArenaFull,
                    0x04 => LoginResponse::LockedOut,
                    0x05 => LoginResponse::PermissionOnly,
                    0x06 => LoginResponse::SpectateOnly,
                    0x07 => LoginResponse::HighPoints,
                    0x08 => LoginResponse::ConnectionSlow,
                    0x09 => LoginResponse::PermissionOnly,
                    0x0A => LoginResponse::ServerFull,
                    0x0B => LoginResponse::InvalidName,
                    0x0C => LoginResponse::OffensiveName,
                    0x0D => LoginResponse::NoBiller,
                    0x0E => LoginResponse::ServerBusy,
                    0x0F => LoginResponse::UsageLow,
                    0x10 => LoginResponse::Restricted,
                    0x11 => LoginResponse::Demo,
                    0x12 => LoginResponse::TooManyDemo,
                    0x13 => LoginResponse::DemoDisabled,
                    0xFF => LoginResponse::Restricted,
                    _ => LoginResponse::Restricted,
                };

                let registration_request = if registration_request != 0 {
                    true
                } else {
                    false
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PasswordResponse(PasswordResponseMessage {
                        response,
                        server_version,
                        registration_request,
                        news_checksum,
                    }),
                )));
            }
            _ => {
                return Err(anyhow!(format!(
                    "invalid game packet type {} received",
                    kind
                )));
            }
        }
    }
}
