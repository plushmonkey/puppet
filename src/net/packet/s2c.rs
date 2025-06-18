use crate::clock::{LocalTick, ServerTick};
use crate::net::packet::Packet;
use crate::net::packet::bi::*;
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
    SyncResponse(SyncResponseMessage),
    Disconnect,
    SmallChunkBody(SmallChunkBodyMessage),
    SmallChunkTail(SmallChunkTailMessage),
    HugeChunk(HugeChunkMessage),
    HugeChunkCancel,
    HugeChunkCancelAck,
    Cluster(ClusterMessage),
}

pub enum GameServerMessage {
    PlayerId(PlayerIdMessage),                 // 0x01
    InGame,                                    // 0x02
    PlayerEntering,                            // 0x03
    PlayerLeaving,                             // 0x04
    LargePosition,                             // 0x05
    PlayerDeath,                               // 0x06
    Chat(ChatMessage),                         // 0x07
    PrizePickup,                               // 0x08
    ScoreUpdate,                               // 0x09
    PasswordResponse(PasswordResponseMessage), // 0x0A
    PowerballGoal,                             // 0x0B
    Voice,                                     // 0x0C
    PlayerFrequencyChange,                     // 0x0D
    TurretLinkCreate,                          // 0x0E
    ArenaSettings,                             // 0x0F
    FileTransfer,                              // 0x10
    Unknown11,                                 // 0x11
    FlagPosition,                              // 0x12
    FlagClaim,                                 // 0x13
    FlagVictory,                               // 0x14
    TurretLinkDestroy,                         // 0x15
    FlagDrop,                                  // 0x16
    Unknown17,                                 // 0x17
    SynchronizationRequest,                    // 0x18
    RequestFile,                               // 0x19
    ResetScore,                                // 0x1A
    ShipReset,                                 // 0x1B
    ForceSpectate,                             // 0x1C
    PlayerTeamAndShipChange,                   // 0x1D
    SelfBannerChanged,                         // 0x1E
    PlayerBannerChanged,                       // 0x1F
    CollectedPrize,                            // 0x20
    BrickDrop,                                 // 0x21
    TurfFlagUpdate,                            // 0x22
    FlagReward,                                // 0x23
    SpeedGameOver,                             // 0x24
    ToggleUfo,                                 // 0x25
    Unknown26,                                 // 0x26
    KeepAlive,                                 // 0x27
    SmallPosition,                             // 0x28
    MapInformation,                            // 0x29
    CompressedMap,                             // 0x2A
    KothSetTimer,                              // 0x2B
    KothReset,                                 // 0x2C
    KothAddTime,                               // 0x2D
    PowerballPosition,                         // 0x2E
    ArenaDirectory,                            // 0x2F
    ZoneBanner,                                // 0x30
    PostLogin,                                 // 0x31
    SetShipCoordinates,                        // 0x32
    CustomLoginMessage,                        // 0x33
    ContinuumVersion,                          // 0x34
    LvzToggle,                                 // 0x35
    LvzModify,                                 // 0x36
    WatchDamageToggle,                         // 0x37
    WatchDamage,                               // 0x38
    BatchedSmallPosition,                      // 0x39
    BatchedLargePosition,                      // 0x3A
    Redirect,                                  // 0x3B
    SelectBox,                                 // 0x3C
}

// Core messages

pub struct EncryptionResponseMessage {
    pub key: u32,
}

// Game messages

// 0x01
pub struct PlayerIdMessage {
    pub id: u16,
}

pub enum ChatKind {
    Arena = 0,
    PublicMacro = 1,
    Public,
    Team,
    Frequency,
    Private,
    Warning,
    RemotePrivate,
    Error,
    Channel,
}

pub struct ChatMessage {
    pub kind: ChatKind,
    pub sound: u8,
    pub sender: u16,
    pub message: String,
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

// 0x0A
pub struct PasswordResponseMessage {
    pub response: LoginResponse,
    pub server_version: u32,
    pub registration_request: bool,
    pub news_checksum: u32,
}

impl ServerMessage {
    pub fn parse(packet: &[u8]) -> Result<Option<ServerMessage>> {
        if packet.len() <= 0 {
            return Err(anyhow!("invalid packet size (0)"));
        }

        let kind = packet[0];

        if kind == 0 {
            return ServerMessage::parse_core_packet(&packet);
        }

        return ServerMessage::parse_game_packet(&packet);
    }

    pub fn parse_core_packet(packet: &[u8]) -> Result<Option<ServerMessage>> {
        if packet.len() < 2 {
            return Err(anyhow!("expected packet type field in core packet"));
        }

        let kind = packet[1];

        match kind {
            0x02 => {
                // EncryptionResponse
                if packet.len() < 6 {
                    return Err(anyhow!("encryption response was too small"));
                }

                let key = u32::from_le_bytes(packet[2..6].try_into().unwrap());

                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::EncryptionResponse(EncryptionResponseMessage { key }),
                )));
            }
            0x03 => {
                // ReliableMessage
                if packet.len() < 7 {
                    return Err(anyhow!("reliable message was too small"));
                }

                let id = u32::from_le_bytes(packet[2..6].try_into().unwrap());
                let data = Packet::new(&packet[6..packet.len()]);

                return Ok(Some(ServerMessage::Core(CoreServerMessage::ReliableData(
                    ReliableDataMessage { id, data },
                ))));
            }
            0x04 => {
                // ReliableAck
                if packet.len() < 6 {
                    return Err(anyhow!("reliable ack was too small"));
                }

                let id = u32::from_le_bytes(packet[2..6].try_into().unwrap());

                return Ok(Some(ServerMessage::Core(CoreServerMessage::ReliableAck(
                    ReliableAckMessage { id },
                ))));
            }
            0x06 => {
                if packet.len() < 10 {
                    return Err(anyhow!("sync response was too small"));
                }

                let request_timestamp =
                    LocalTick::new(u32::from_le_bytes(packet[2..6].try_into().unwrap()));
                let server_timestamp =
                    ServerTick::new(u32::from_le_bytes(packet[6..10].try_into().unwrap()), 0);

                return Ok(Some(ServerMessage::Core(CoreServerMessage::SyncResponse(
                    SyncResponseMessage {
                        request_timestamp,
                        server_timestamp,
                    },
                ))));
            }
            0x07 => {
                return Ok(Some(ServerMessage::Core(CoreServerMessage::Disconnect)));
            }
            0x08 => {
                let data = Packet::new(&packet[2..packet.len()]);

                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::SmallChunkBody(SmallChunkBodyMessage { data }),
                )));
            }
            0x09 => {
                let data = Packet::new(&packet[2..packet.len()]);

                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::SmallChunkTail(SmallChunkTailMessage { data }),
                )));
            }
            0x0A => {
                let total_size = u32::from_le_bytes(packet[2..6].try_into().unwrap());
                let data = Packet::new(&packet[6..]);

                return Ok(Some(ServerMessage::Core(CoreServerMessage::HugeChunk(
                    HugeChunkMessage { total_size, data },
                ))));
            }
            0x0B => {
                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::HugeChunkCancel,
                )));
            }
            0x0C => {
                return Ok(Some(ServerMessage::Core(
                    CoreServerMessage::HugeChunkCancelAck,
                )));
            }
            0x0E => {
                let data = Packet::new(&packet[2..packet.len()]);

                return Ok(Some(ServerMessage::Core(CoreServerMessage::Cluster(
                    ClusterMessage { data },
                ))));
            }
            _ => {
                return Err(anyhow!(format!(
                    "invalid core packet type {} received",
                    kind
                )));
            }
        }
    }

    pub fn parse_game_packet(packet: &[u8]) -> Result<Option<ServerMessage>> {
        let kind = packet[0];

        match kind {
            0x01 => {
                if packet.len() < 3 {
                    return Err(anyhow!("player id message was too small"));
                }

                let id = u16::from_le_bytes(packet[1..3].try_into().unwrap());

                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerId(
                    PlayerIdMessage { id },
                ))));
            }
            0x02 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::InGame)));
            }
            0x03 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerEntering)));
            }
            0x04 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerLeaving)));
            }
            0x05 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::LargePosition)));
            }
            0x06 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerDeath)));
            }
            0x07 => {
                if packet.len() < 5 {
                    return Err(anyhow!("chat message was too small"));
                }

                let kind = match packet[1] {
                    0x00 => ChatKind::Arena,
                    0x01 => ChatKind::PublicMacro,
                    0x02 => ChatKind::Public,
                    0x03 => ChatKind::Team,
                    0x04 => ChatKind::Frequency,
                    0x05 => ChatKind::Private,
                    0x06 => ChatKind::Warning,
                    0x07 => ChatKind::RemotePrivate,
                    0x08 => ChatKind::Error,
                    0x09 => ChatKind::Channel,
                    _ => ChatKind::Arena,
                };

                let sound = packet[2];
                let sender = u16::from_le_bytes(packet[3..5].try_into().unwrap());
                let message = std::str::from_utf8(&packet[5..]);
                if let Err(e) = message {
                    return Err(anyhow!(e));
                }

                let chat = ChatMessage {
                    kind,
                    sound,
                    sender,
                    message: message.unwrap().to_owned(),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::Chat(chat))));
            }
            0x08 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PrizePickup)));
            }
            0x09 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ScoreUpdate)));
            }
            0x0A => {
                if packet.len() < 28 {
                    return Err(anyhow!("password packet message was too small"));
                }

                let response_type = packet[1];
                let server_version = u32::from_le_bytes(packet[2..6].try_into().unwrap());
                let registration_request = packet[19];
                let news_checksum = u32::from_le_bytes(packet[24..28].try_into().unwrap());

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
            0x0B => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PowerballGoal)));
            }
            0x0C => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Voice)));
            }
            0x0D => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerFrequencyChange,
                )));
            }
            0x0E => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::TurretLinkCreate,
                )));
            }
            0x0F => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ArenaSettings)));
            }
            0x10 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FileTransfer)));
            }
            0x11 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown11)));
            }
            0x12 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagPosition)));
            }
            0x13 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagClaim)));
            }
            0x14 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagVictory)));
            }
            0x15 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::TurretLinkDestroy,
                )));
            }
            0x16 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagDrop)));
            }
            0x17 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown17)));
            }
            0x18 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SynchronizationRequest,
                )));
            }
            0x19 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::RequestFile)));
            }
            0x1A => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ResetScore)));
            }
            0x1B => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ShipReset)));
            }
            0x1C => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ForceSpectate)));
            }
            0x1D => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerTeamAndShipChange,
                )));
            }
            0x1E => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SelfBannerChanged,
                )));
            }
            0x1F => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerBannerChanged,
                )));
            }
            0x20 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::CollectedPrize)));
            }
            0x21 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::BrickDrop)));
            }
            0x22 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::TurfFlagUpdate)));
            }
            0x23 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagReward)));
            }
            0x24 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::SpeedGameOver)));
            }
            0x25 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ToggleUfo)));
            }
            0x26 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown26)));
            }
            0x27 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::KeepAlive)));
            }
            0x28 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::SmallPosition)));
            }
            0x29 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::MapInformation)));
            }
            0x2A => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::CompressedMap)));
            }
            0x2B => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::KothSetTimer)));
            }
            0x2C => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::KothReset)));
            }
            0x2D => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::KothAddTime)));
            }
            0x2E => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PowerballPosition,
                )));
            }
            0x2F => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ArenaDirectory)));
            }
            0x30 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ZoneBanner)));
            }
            0x31 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PostLogin)));
            }
            0x32 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SetShipCoordinates,
                )));
            }
            0x33 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::CustomLoginMessage,
                )));
            }
            0x34 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::ContinuumVersion,
                )));
            }
            0x35 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::LvzToggle)));
            }
            0x36 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::LvzModify)));
            }
            0x37 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::WatchDamageToggle,
                )));
            }
            0x38 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::WatchDamage)));
            }
            0x39 => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::BatchedSmallPosition,
                )));
            }
            0x3A => {
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::BatchedLargePosition,
                )));
            }
            0x3B => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Redirect)));
            }
            0x3C => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::SelectBox)));
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
