use crate::arena_settings::ArenaSettings;
use crate::clock::{LocalTick, ServerTick};
use crate::net::packet::Packet;
use crate::net::packet::bi::*;
use crate::ship::Ship;
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
    PlayerId(PlayerIdMessage),                               // 0x01
    InGame,                                                  // 0x02
    PlayerEntering(PlayerEnteringMessage),                   // 0x03
    PlayerLeaving(PlayerLeavingMessage),                     // 0x04
    LargePosition(LargePositionMessage),                     // 0x05
    PlayerDeath(PlayerDeathMessage),                         // 0x06
    Chat(ChatMessage),                                       // 0x07
    PrizePickup(PrizePickupMessage),                         // 0x08
    ScoreUpdate(ScoreUpdateMessage),                         // 0x09
    PasswordResponse(PasswordResponseMessage),               // 0x0A
    PowerballGoal(PowerballGoalMessage),                     // 0x0B
    Voice(VoiceMessage),                                     // 0x0C
    PlayerFrequencyChange(PlayerFrequencyChangeMessage),     // 0x0D
    TurretLinkCreate(TurretLinkCreateMessage),               // 0x0E
    ArenaSettings(Box<ArenaSettings>),                       // 0x0F
    FileTransfer(FileTransferMessage),                       // 0x10
    Unknown11,                                               // 0x11
    FlagPosition(FlagPositionMessage),                       // 0x12
    FlagClaim(FlagClaimMessage),                             // 0x13
    FlagVictory(FlagVictoryMessage),                         // 0x14
    TurretLinkDestroy(TurretLinkDestroyMessage),             // 0x15
    FlagDrop(FlagDropMessage),                               // 0x16
    Unknown17,                                               // 0x17
    SynchronizationRequest(SynchronizationRequestMessage),   // 0x18
    RequestFile(RequestFileMessage),                         // 0x19
    ResetScore(ResetScoreMessage),                           // 0x1A
    ShipReset,                                               // 0x1B
    SpectateData(SpectateDataMessage),                       // 0x1C
    PlayerTeamAndShipChange(PlayerTeamAndShipChangeMessage), // 0x1D
    SelfBannerChanged(SelfBannerChangedMessage),             // 0x1E
    PlayerBannerChanged(PlayerBannerChangedMessage),         // 0x1F
    CollectedPrize(CollectedPrizeMessage),                   // 0x20
    BrickDrop(BrickDropMessage),                             // 0x21
    TurfFlagUpdate(TurfFlagUpdateMessage),                   // 0x22
    FlagReward(FlagRewardMessage),                           // 0x23
    SpeedGameOver(SpeedGameOverMessage),                     // 0x24
    ToggleUfo(ToggleUfoMessage),                             // 0x25
    Unknown26,                                               // 0x26
    KeepAlive,                                               // 0x27
    SmallPosition(SmallPositionMessage),                     // 0x28
    MapInformation(MapInformationMessage),                   // 0x29
    CompressedMap(CompressedMapMessage),                     // 0x2A
    KothSetTimer(KothSetTimerMessage),                       // 0x2B
    KothReset(KothResetMessage),                             // 0x2C
    KothAddTime(KothAddTimeMessage),                         // 0x2D
    PowerballPosition(PowerballPositionMessage),             // 0x2E
    ArenaDirectory(ArenaDirectoryMessage),                   // 0x2F
    ZoneBanner(ZoneBannerMessage),                           // 0x30
    PostLogin,                                               // 0x31
    SetShipCoordinates(SetShipCoordinatesMessage),           // 0x32
    CustomLoginFailure(CustomLoginFailureMessage),           // 0x33
    ContinuumVersion(ContinuumVersionMessage),               // 0x34
    LvzToggle,                                               // 0x35
    LvzModify,                                               // 0x36
    WatchDamageToggle,                                       // 0x37
    WatchDamage,                                             // 0x38
    BatchedSmallPosition,                                    // 0x39
    BatchedLargePosition,                                    // 0x3A
    Redirect,                                                // 0x3B
    SelectBox,                                               // 0x3C
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

pub struct PlayerEntering {
    pub ship: Ship,
    pub name: String,
    pub squad: String,
    pub kill_points: u32,
    pub flag_points: u32,
    pub player_id: u16,
    pub frequency: u16,
    pub kills: u16,
    pub deaths: u16,
    pub attach_parent: u16,
    pub flag_count: u16,
    pub has_koth: bool,
}

// 0x03
pub struct PlayerEnteringMessage {
    pub players: Vec<PlayerEntering>,
}

// 0x04
pub struct PlayerLeavingMessage {
    pub player_id: u16,
}

pub struct ItemSet {
    pub shield_active: bool,
    pub super_active: bool,
    pub bursts: u8,
    pub repels: u8,
    pub thors: u8,
    pub bricks: u8,
    pub decoys: u8,
    pub rockets: u8,
    pub portals: u8,
}

impl ItemSet {
    pub fn empty() -> ItemSet {
        ItemSet {
            shield_active: false,
            super_active: false,
            bursts: 0,
            repels: 0,
            thors: 0,
            bricks: 0,
            decoys: 0,
            rockets: 0,
            portals: 0,
        }
    }

    pub fn parse(data: u32) -> ItemSet {
        ItemSet {
            shield_active: (data & 1) != 0,
            super_active: ((data >> 1) & 1) != 0,
            bursts: ((data >> 2) & 0x0F) as u8,
            repels: ((data >> 6) & 0x0F) as u8,
            thors: ((data >> 10) & 0x0F) as u8,
            bricks: ((data >> 14) & 0x0F) as u8,
            decoys: ((data >> 18) & 0x0F) as u8,
            rockets: ((data >> 22) & 0x0F) as u8,
            portals: ((data >> 26) & 0x0F) as u8,
        }
    }
}

pub struct ExtraPositionData {
    pub energy: u16,
    pub s2c_lag: u16,
    pub timer: u16,
    pub items: ItemSet,
}

// 0x05
pub struct LargePositionMessage {
    pub direction: u8,
    pub timestamp: u16,
    pub x: u16,
    pub y: u16,
    pub x_velocity: i16,
    pub y_velocity: i16,
    pub player_id: u16,
    pub checksum: u8,
    pub status: u8,
    pub ping: u8,
    pub bounty: u16,
    pub weapon: u16,
    pub extra: Option<ExtraPositionData>,
}

// 0x06
pub struct PlayerDeathMessage {
    pub prize_id: i8,
    pub killer_id: u16,
    pub killed_id: u16,
    pub bounty: u16,
    pub flag_transfer: u16,
}

#[derive(Copy, Clone)]
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

// 0x07
pub struct ChatMessage {
    pub kind: ChatKind,
    pub sound: u8,
    pub sender: u16,
    pub message: String,
}

// 0x08
pub struct PrizePickupMessage {
    pub timestamp: ServerTick,
    pub x: u16,
    pub y: u16,
    pub prize_id: i16,
    pub player_id: u16,
}

// 0x09
pub struct ScoreUpdateMessage {
    pub player_id: u16,
    pub kill_points: u32,
    pub flag_points: u32,
    pub kills: u16,
    pub deaths: u16,
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

// 0x0B
pub struct PowerballGoalMessage {
    pub frequency: u16,
    pub team_points: u32,
}

// 0x0C
pub struct VoiceMessage {
    pub player_id: u16,
    pub wav_data: Vec<u8>,
}

// 0x0D
pub struct PlayerFrequencyChangeMessage {
    pub player_id: u16,
    pub frequency: u16,
}

// 0x0E
pub struct TurretLinkCreateMessage {
    pub requester_id: u16,
    pub destination_id: Option<u16>,
}

// 0x10
pub struct FileTransferMessage {
    pub filename: String,
    pub data: Vec<u8>,
}

// 0x12
pub struct FlagPositionMessage {
    pub flag_id: u16,
    pub x: u16,
    pub y: u16,
    pub owner_freq: u16,
}

// 0x13
pub struct FlagClaimMessage {
    pub flag_id: u16,
    pub player_id: u16,
}

// 0x14
pub struct FlagVictoryMessage {
    pub frequency: u16,
    pub points: u32,
}

// 0x15
pub struct TurretLinkDestroyMessage {
    pub player_id: u16,
}

// 0x16
pub struct FlagDropMessage {
    pub player_id: u16,
}

// 0x18
pub struct SynchronizationRequestMessage {
    pub prize_seed: u32,
    pub door_seed: u32,
    pub timestamp: ServerTick,
    pub checksum_key: u32,
}

// 0x19
pub struct RequestFileMessage {
    pub local_filename: String,
    pub remote_filename: String,
}

// 0x1A
pub struct ResetScoreMessage {
    pub player_id: u16,
}

// 0x1C
pub enum SpectateDataMessage {
    Player(u16),
    ExtraPositionInfo(bool),
}

// 0x1D
pub struct PlayerTeamAndShipChangeMessage {
    pub ship: Ship,
    pub player_id: u16,
    pub frequency: u16,
}

// 0x1E
pub struct SelfBannerChangedMessage {
    pub enabled: bool,
}

// 0x1F
pub struct PlayerBannerChangedMessage {
    pub player_id: u16,
    pub banner_data: [u8; 96],
}

// 0x20
pub struct CollectedPrizeMessage {
    pub count: u16,
    pub prize_id: i16,
}

// 0x21
pub struct BrickDropMessage {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub frequency: u16,
    pub brick_id: u16,
    pub timestamp: ServerTick,
}

// 0x22
pub struct TurfFlagUpdateMessage {
    pub flag_teams: Vec<u16>,
}

pub struct FlagReward {
    pub frequency: u16,
    pub points: u16,
}

// 0x23
pub struct FlagRewardMessage {
    pub rewards: Vec<FlagReward>,
}

// 0x24
pub struct SpeedGameOverMessage {
    pub best_recorded_game: bool,
    pub rank: u16,
    pub score: u32,
    pub player1_score: u32,
    pub player2_score: u32,
    pub player3_score: u32,
    pub player4_score: u32,
    pub player5_score: u32,
    pub player1_id: u16,
    pub player2_id: u16,
    pub player3_id: u16,
    pub player4_id: u16,
    pub player5_id: u16,
}

// 0x25
pub struct ToggleUfoMessage {
    pub enable: bool,
}

// 0x28
pub struct SmallPositionMessage {
    pub direction: u8,
    pub timestamp: u16,
    pub x: u16,
    pub y: u16,
    pub x_velocity: i16,
    pub y_velocity: i16,
    pub ping: u8,
    pub bounty: u8,
    pub player_id: u8,
    pub status: u8,
    pub extra: Option<ExtraPositionData>,
}

// 0x29
pub struct MapInformationMessage {
    pub filename: String,
    pub checksum: u32,
    pub filesize: Option<u32>,
}

// 0x2A
pub struct CompressedMapMessage {
    pub filename: String,
    pub data: Vec<u8>,
}

// 0x2B
pub struct KothSetTimerMessage {
    pub timer: u32,
}

// 0x2C
pub struct KothResetMessage {
    pub add_crown: bool,
    pub timer: u32,
    pub player_id: u16,
}

// 0x2D
pub struct KothAddTimeMessage {
    pub added_time: u32,
}

// 0x2E
pub struct PowerballPositionMessage {
    pub ball_id: u8,
    pub x: u16,
    pub y: u16,
    pub x_velocity: i16,
    pub y_velocity: i16,
    pub owner_id: u16,
    pub timestamp: ServerTick,
}

#[derive(Debug)]
pub struct ArenaDirectoryEntry {
    pub name: String,
    pub count: u16,
    pub current: bool,
}

// 0x2F
#[derive(Debug)]
pub struct ArenaDirectoryMessage {
    pub entries: Vec<ArenaDirectoryEntry>,
}

// 0x30
pub struct ZoneBannerMessage {
    pub display_mode: u8,
    pub width: u16,
    pub height: u16,
    pub duration: u32,
    pub data: Vec<u8>,
}

// 0x32
pub struct SetShipCoordinatesMessage {
    pub x: u16,
    pub y: u16,
}

// 0x33
pub struct CustomLoginFailureMessage {
    pub reason: String,
}

// 0x34
pub struct ContinuumVersionMessage {
    pub version: u16,
    pub checksum: u32,
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
                let mut entering_message = PlayerEnteringMessage {
                    players: Vec::new(),
                };

                let mut data = packet;
                while data.len() >= 64 {
                    let name = std::str::from_utf8(&data[3..23])?;
                    let squad = std::str::from_utf8(&data[23..43])?;

                    let current = PlayerEntering {
                        ship: Ship::from_network_value(data[0]),
                        name: name.to_owned(),
                        squad: squad.to_owned(),
                        kill_points: u32::from_le_bytes(data[43..47].try_into().unwrap()),
                        flag_points: u32::from_le_bytes(data[47..51].try_into().unwrap()),
                        player_id: u16::from_le_bytes(data[51..53].try_into().unwrap()),
                        frequency: u16::from_le_bytes(data[53..55].try_into().unwrap()),
                        kills: u16::from_le_bytes(data[55..57].try_into().unwrap()),
                        deaths: u16::from_le_bytes(data[57..59].try_into().unwrap()),
                        attach_parent: u16::from_le_bytes(data[59..61].try_into().unwrap()),
                        flag_count: u16::from_le_bytes(data[61..63].try_into().unwrap()),
                        has_koth: data[63] != 0,
                    };

                    entering_message.players.push(current);
                    data = &data[64..];
                }

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerEntering(entering_message),
                )));
            }
            0x04 => {
                if packet.len() < 3 {
                    return Err(anyhow!("player leaving packet too small"));
                }

                let message = PlayerLeavingMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerLeaving(
                    message,
                ))));
            }
            0x05 => {
                let mut extra = None;
                if packet.len() > 20 {
                    let mut energy = 0;
                    let mut s2c_lag = 0;
                    let mut timer = 0;
                    let mut items = ItemSet::empty();

                    if packet.len() >= 23 {
                        energy = u16::from_le_bytes(packet[21..23].try_into().unwrap());
                    }

                    if packet.len() >= 25 {
                        s2c_lag = u16::from_le_bytes(packet[23..25].try_into().unwrap());
                    }

                    if packet.len() >= 27 {
                        timer = u16::from_le_bytes(packet[25..27].try_into().unwrap());
                    }

                    if packet.len() >= 31 {
                        let item_bytes = u32::from_le_bytes(packet[27..31].try_into().unwrap());
                        items = ItemSet::parse(item_bytes);
                    }

                    extra = Some(ExtraPositionData {
                        energy,
                        s2c_lag,
                        timer,
                        items,
                    });
                }
                let position = LargePositionMessage {
                    direction: packet[1],
                    timestamp: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    x: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                    y_velocity: i16::from_le_bytes(packet[6..8].try_into().unwrap()),
                    player_id: u16::from_le_bytes(packet[8..10].try_into().unwrap()),
                    x_velocity: i16::from_le_bytes(packet[10..12].try_into().unwrap()),
                    checksum: packet[12],
                    status: packet[13],
                    ping: packet[14],
                    y: u16::from_le_bytes(packet[15..17].try_into().unwrap()),
                    bounty: u16::from_le_bytes(packet[17..19].try_into().unwrap()),
                    weapon: u16::from_le_bytes(packet[19..21].try_into().unwrap()),
                    extra,
                };
                return Ok(Some(ServerMessage::Game(GameServerMessage::LargePosition(
                    position,
                ))));
            }
            0x06 => {
                if packet.len() < 10 {
                    return Err(anyhow!("player death message was too small"));
                }

                let player_death = PlayerDeathMessage {
                    prize_id: packet[1] as i8,
                    killer_id: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    killed_id: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                    bounty: u16::from_le_bytes(packet[6..8].try_into().unwrap()),
                    flag_transfer: u16::from_le_bytes(packet[8..10].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::PlayerDeath(
                    player_death,
                ))));
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
                if packet.len() < 13 {
                    return Err(anyhow!("prize pickup message was too small"));
                }

                let message = PrizePickupMessage {
                    timestamp: ServerTick::new(
                        u32::from_le_bytes(packet[1..5].try_into().unwrap()),
                        0,
                    ),
                    x: u16::from_le_bytes(packet[5..7].try_into().unwrap()),
                    y: u16::from_le_bytes(packet[7..9].try_into().unwrap()),
                    prize_id: i16::from_le_bytes(packet[9..11].try_into().unwrap()),
                    player_id: u16::from_le_bytes(packet[11..13].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::PrizePickup(
                    message,
                ))));
            }
            0x09 => {
                if packet.len() < 15 {
                    return Err(anyhow!("score update message was too small"));
                }

                let message = ScoreUpdateMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    kill_points: u32::from_le_bytes(packet[3..7].try_into().unwrap()),
                    flag_points: u32::from_le_bytes(packet[7..11].try_into().unwrap()),
                    kills: u16::from_le_bytes(packet[11..13].try_into().unwrap()),
                    deaths: u16::from_le_bytes(packet[13..15].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::ScoreUpdate(
                    message,
                ))));
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
                if packet.len() < 7 {
                    return Err(anyhow!("powerball goal message was too small"));
                }

                let message = PowerballGoalMessage {
                    frequency: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    team_points: u32::from_le_bytes(packet[3..7].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::PowerballGoal(
                    message,
                ))));
            }
            0x0C => {
                if packet.len() < 3 {
                    return Err(anyhow!("voice message was too small"));
                }

                let wav_data = packet[3..].to_vec();

                let message = VoiceMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    wav_data,
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::Voice(message))));
            }
            0x0D => {
                if packet.len() < 5 {
                    return Err(anyhow!("player frqeuency change message was too small"));
                }

                let message = PlayerFrequencyChangeMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    frequency: u16::from_le_bytes(packet[3..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerFrequencyChange(message),
                )));
            }
            0x0E => {
                if packet.len() < 3 {
                    return Err(anyhow!("turret link create message was too small"));
                }

                let mut destination_id = None;
                if packet.len() >= 5 {
                    destination_id = Some(u16::from_le_bytes(packet[3..5].try_into().unwrap()));
                }

                let message = TurretLinkCreateMessage {
                    requester_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    destination_id,
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::TurretLinkCreate(message),
                )));
            }
            0x0F => {
                let settings = ArenaSettings::parse(packet);
                if let None = settings {
                    return Err(anyhow!("arena settings message was too small"));
                }
                return Ok(Some(ServerMessage::Game(GameServerMessage::ArenaSettings(
                    Box::new(settings.unwrap()),
                ))));
            }
            0x10 => {
                if packet.len() < 17 {
                    return Err(anyhow!("file transfer message was too small"));
                }

                let filename = std::str::from_utf8(&packet[1..17])?;
                let data = packet[17..].to_vec();

                let message = FileTransferMessage {
                    filename: filename.to_owned(),
                    data,
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FileTransfer(
                    message,
                ))));
            }
            0x11 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown11)));
            }
            0x12 => {
                if packet.len() < 9 {
                    return Err(anyhow!("flag position message was too small"));
                }

                let message = FlagPositionMessage {
                    flag_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    x: u16::from_le_bytes(packet[3..5].try_into().unwrap()),
                    y: u16::from_le_bytes(packet[5..7].try_into().unwrap()),
                    owner_freq: u16::from_le_bytes(packet[7..9].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagPosition(
                    message,
                ))));
            }
            0x13 => {
                if packet.len() < 5 {
                    return Err(anyhow!("flag claim message was too small"));
                }

                let message = FlagClaimMessage {
                    flag_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    player_id: u16::from_le_bytes(packet[3..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagClaim(
                    message,
                ))));
            }
            0x14 => {
                if packet.len() < 7 {
                    return Err(anyhow!("flag victory message was too small"));
                }

                let message = FlagVictoryMessage {
                    frequency: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    points: u32::from_le_bytes(packet[3..7].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagVictory(
                    message,
                ))));
            }
            0x15 => {
                if packet.len() < 3 {
                    return Err(anyhow!("turret link destroy message too small"));
                }

                let message = TurretLinkDestroyMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::TurretLinkDestroy(message),
                )));
            }
            0x16 => {
                if packet.len() < 3 {
                    return Err(anyhow!("flag drop message too small"));
                }

                let message = FlagDropMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagDrop(
                    message,
                ))));
            }
            0x17 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown17)));
            }
            0x18 => {
                if packet.len() < 17 {
                    return Err(anyhow!("synchronization request message was too small"));
                }

                let message = SynchronizationRequestMessage {
                    prize_seed: u32::from_le_bytes(packet[1..5].try_into().unwrap()),
                    door_seed: u32::from_le_bytes(packet[5..9].try_into().unwrap()),
                    timestamp: ServerTick::new(
                        u32::from_le_bytes(packet[9..13].try_into().unwrap()),
                        0,
                    ),
                    checksum_key: u32::from_le_bytes(packet[13..17].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SynchronizationRequest(message),
                )));
            }
            0x19 => {
                if packet.len() < 274 {
                    return Err(anyhow!("request file message was too small"));
                }

                let local_filename = std::str::from_utf8(&packet[1..257])?;
                let remote_filename = std::str::from_utf8(&packet[257..274])?;

                let message = RequestFileMessage {
                    local_filename: local_filename.to_owned(),
                    remote_filename: remote_filename.to_owned(),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::RequestFile(
                    message,
                ))));
            }
            0x1A => {
                if packet.len() < 3 {
                    return Err(anyhow!("reset score message was too small"));
                }

                let message = ResetScoreMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::ResetScore(
                    message,
                ))));
            }
            0x1B => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::ShipReset)));
            }
            0x1C => {
                if packet.len() <= 1 {
                    return Err(anyhow!("spectate data message was too small"));
                }

                if packet.len() == 2 {
                    let message = SpectateDataMessage::ExtraPositionInfo(packet[1] != 0);
                    return Ok(Some(ServerMessage::Game(GameServerMessage::SpectateData(
                        message,
                    ))));
                }

                let message = SpectateDataMessage::Player(u16::from_le_bytes(
                    packet[1..3].try_into().unwrap(),
                ));

                return Ok(Some(ServerMessage::Game(GameServerMessage::SpectateData(
                    message,
                ))));
            }
            0x1D => {
                if packet.len() < 6 {
                    return Err(anyhow!("player team and ship change message was too small"));
                }

                let message = PlayerTeamAndShipChangeMessage {
                    ship: Ship::from_network_value(packet[1]),
                    player_id: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    frequency: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                };
                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerTeamAndShipChange(message),
                )));
            }
            0x1E => {
                if packet.len() < 2 {
                    return Err(anyhow!("self banner changed message was too small"));
                }

                let message = SelfBannerChangedMessage {
                    enabled: packet[1] != 0,
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SelfBannerChanged(message),
                )));
            }
            0x1F => {
                if packet.len() < 99 {
                    return Err(anyhow!("player banner changed message was too small"));
                }

                let message = PlayerBannerChangedMessage {
                    player_id: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    banner_data: packet[3..99].try_into().unwrap(),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PlayerBannerChanged(message),
                )));
            }
            0x20 => {
                if packet.len() < 5 {
                    return Err(anyhow!("collected prize message was too small"));
                }

                let message = CollectedPrizeMessage {
                    count: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    prize_id: i16::from_le_bytes(packet[3..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::CollectedPrize(message),
                )));
            }
            0x21 => {
                if packet.len() < 17 {
                    return Err(anyhow!("brick drop message was too small"));
                }

                let message = BrickDropMessage {
                    x1: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    y1: u16::from_le_bytes(packet[3..5].try_into().unwrap()),
                    x2: u16::from_le_bytes(packet[5..7].try_into().unwrap()),
                    y2: u16::from_le_bytes(packet[7..9].try_into().unwrap()),
                    frequency: u16::from_le_bytes(packet[9..11].try_into().unwrap()),
                    brick_id: u16::from_le_bytes(packet[11..13].try_into().unwrap()),
                    timestamp: ServerTick::new(
                        u32::from_le_bytes(packet[13..17].try_into().unwrap()),
                        0,
                    ),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::BrickDrop(
                    message,
                ))));
            }
            0x22 => {
                if packet.len() < 3 {
                    return Err(anyhow!("turf flag update message was too small"));
                }

                let mut flag_teams = Vec::new();
                let mut data = &packet[1..];

                while data.len() >= 2 {
                    let team = u16::from_le_bytes(data[..2].try_into().unwrap());
                    flag_teams.push(team);
                    data = &data[2..];
                }

                let message = TurfFlagUpdateMessage { flag_teams };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::TurfFlagUpdate(message),
                )));
            }
            0x23 => {
                if packet.len() < 5 {
                    return Err(anyhow!("flag reward message too small"));
                }

                let mut rewards = Vec::new();
                let mut data = &packet[1..];

                while data.len() >= 4 {
                    let reward = FlagReward {
                        frequency: u16::from_le_bytes(packet[..2].try_into().unwrap()),
                        points: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    };

                    rewards.push(reward);

                    data = &data[4..];
                }

                let message = FlagRewardMessage { rewards };

                return Ok(Some(ServerMessage::Game(GameServerMessage::FlagReward(
                    message,
                ))));
            }
            0x24 => {
                if packet.len() < 38 {
                    return Err(anyhow!("speed game over message was too small"));
                }

                let message = SpeedGameOverMessage {
                    best_recorded_game: packet[1] != 0,
                    rank: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    score: u32::from_le_bytes(packet[4..8].try_into().unwrap()),
                    player1_score: u32::from_le_bytes(packet[8..12].try_into().unwrap()),
                    player2_score: u32::from_le_bytes(packet[12..16].try_into().unwrap()),
                    player3_score: u32::from_le_bytes(packet[16..20].try_into().unwrap()),
                    player4_score: u32::from_le_bytes(packet[20..24].try_into().unwrap()),
                    player5_score: u32::from_le_bytes(packet[24..28].try_into().unwrap()),
                    player1_id: u16::from_le_bytes(packet[28..30].try_into().unwrap()),
                    player2_id: u16::from_le_bytes(packet[30..32].try_into().unwrap()),
                    player3_id: u16::from_le_bytes(packet[32..34].try_into().unwrap()),
                    player4_id: u16::from_le_bytes(packet[34..36].try_into().unwrap()),
                    player5_id: u16::from_le_bytes(packet[36..38].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::SpeedGameOver(
                    message,
                ))));
            }
            0x25 => {
                if packet.len() < 2 {
                    return Err(anyhow!("toggle ufo message was too small"));
                }

                let message = ToggleUfoMessage {
                    enable: packet[1] != 0,
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::ToggleUfo(
                    message,
                ))));
            }
            0x26 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::Unknown26)));
            }
            0x27 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::KeepAlive)));
            }
            0x28 => {
                if packet.len() < 16 {
                    return Err(anyhow!("small position message was too small"));
                }

                let mut extra = None;

                if packet.len() > 16 {
                    let mut energy = 0;
                    let mut s2c_lag = 0;
                    let mut timer = 0;
                    let mut items = ItemSet::empty();

                    if packet.len() >= 18 {
                        energy = u16::from_le_bytes(packet[16..18].try_into().unwrap());
                    }

                    if packet.len() >= 20 {
                        s2c_lag = u16::from_le_bytes(packet[18..20].try_into().unwrap());
                    }

                    if packet.len() >= 22 {
                        timer = u16::from_le_bytes(packet[20..22].try_into().unwrap());
                    }

                    if packet.len() >= 26 {
                        let item_bytes = u32::from_le_bytes(packet[22..26].try_into().unwrap());
                        items = ItemSet::parse(item_bytes);
                    }

                    extra = Some(ExtraPositionData {
                        energy,
                        s2c_lag,
                        timer,
                        items,
                    });
                }

                let message = SmallPositionMessage {
                    direction: packet[1],
                    timestamp: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    x: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                    ping: packet[6],
                    bounty: packet[7],
                    player_id: packet[8],
                    status: packet[9],
                    y_velocity: i16::from_le_bytes(packet[10..12].try_into().unwrap()),
                    y: u16::from_le_bytes(packet[12..14].try_into().unwrap()),
                    x_velocity: i16::from_le_bytes(packet[14..16].try_into().unwrap()),
                    extra,
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::SmallPosition(
                    message,
                ))));
            }
            0x29 => {
                if packet.len() < 21 {
                    return Err(anyhow!("map information message was too small"));
                }

                let filename = std::str::from_utf8(&packet[1..17])?;
                let mut filesize = None;

                if packet.len() >= 25 {
                    filesize = Some(u32::from_le_bytes(packet[21..25].try_into().unwrap()));
                }

                let message = MapInformationMessage {
                    filename: filename.to_owned(),
                    checksum: u32::from_le_bytes(packet[17..21].try_into().unwrap()),
                    filesize,
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::MapInformation(message),
                )));
            }
            0x2A => {
                if packet.len() < 17 {
                    return Err(anyhow!("compressed map message was too small"));
                }

                let filename = std::str::from_utf8(&packet[1..17])?;
                let data = packet[17..].to_vec();

                let message = CompressedMapMessage {
                    filename: filename.to_owned(),
                    data,
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::CompressedMap(
                    message,
                ))));
            }
            0x2B => {
                if packet.len() < 5 {
                    return Err(anyhow!("koth set timer message was too small"));
                }

                let message = KothSetTimerMessage {
                    timer: u32::from_le_bytes(packet[1..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::KothSetTimer(
                    message,
                ))));
            }
            0x2C => {
                if packet.len() < 8 {
                    return Err(anyhow!("koth reset message was too small"));
                }

                let message = KothResetMessage {
                    add_crown: packet[1] != 0,
                    timer: u32::from_le_bytes(packet[2..6].try_into().unwrap()),
                    player_id: u16::from_le_bytes(packet[6..8].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::KothReset(
                    message,
                ))));
            }
            0x2D => {
                if packet.len() < 5 {
                    return Err(anyhow!("koth add time message was too small"));
                }

                let message = KothAddTimeMessage {
                    added_time: u32::from_le_bytes(packet[1..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::KothAddTime(
                    message,
                ))));
            }
            0x2E => {
                if packet.len() < 16 {
                    return Err(anyhow!("powerball position message was too small"));
                }

                let message = PowerballPositionMessage {
                    ball_id: packet[1],
                    x: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    y: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                    x_velocity: i16::from_le_bytes(packet[6..8].try_into().unwrap()),
                    y_velocity: i16::from_le_bytes(packet[8..10].try_into().unwrap()),
                    owner_id: u16::from_le_bytes(packet[10..12].try_into().unwrap()),
                    timestamp: ServerTick::new(
                        u32::from_le_bytes(packet[12..16].try_into().unwrap()),
                        0,
                    ),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::PowerballPosition(message),
                )));
            }
            0x2F => {
                let mut entries = Vec::new();
                let mut data = &packet[1..];

                while data.len() >= 3 {
                    let name_end = data.iter().position(|v| *v == 0);

                    if let None = name_end {
                        return Err(anyhow!("invalid arena name in arena directory entry"));
                    }

                    let name_end = name_end.unwrap() + 1;
                    let name = std::str::from_utf8(&data[..name_end - 1])?;

                    if data.len() < name_end + 2 {
                        return Err(anyhow!("invalid count in arena directory entry"));
                    }

                    let mut count =
                        i16::from_le_bytes(data[name_end..name_end + 2].try_into().unwrap());
                    let current = count < 0;

                    if count < 0 {
                        count *= -1;
                    }

                    let entry = ArenaDirectoryEntry {
                        name: name.to_owned(),
                        count: count as u16,
                        current,
                    };

                    entries.push(entry);
                    data = &data[name_end + 2..];
                }

                let message = ArenaDirectoryMessage { entries };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::ArenaDirectory(message),
                )));
            }
            0x30 => {
                if packet.len() < 10 {
                    return Err(anyhow!("zone banner message was too small"));
                }

                let message = ZoneBannerMessage {
                    display_mode: packet[1],
                    width: u16::from_le_bytes(packet[2..4].try_into().unwrap()),
                    height: u16::from_le_bytes(packet[4..6].try_into().unwrap()),
                    duration: u32::from_le_bytes(packet[6..10].try_into().unwrap()),
                    data: packet[10..].to_vec(),
                };

                return Ok(Some(ServerMessage::Game(GameServerMessage::ZoneBanner(
                    message,
                ))));
            }
            0x31 => {
                return Ok(Some(ServerMessage::Game(GameServerMessage::PostLogin)));
            }
            0x32 => {
                if packet.len() < 5 {
                    return Err(anyhow!("set ship coordinates message was too small"));
                }

                let message = SetShipCoordinatesMessage {
                    x: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    y: u16::from_le_bytes(packet[3..5].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::SetShipCoordinates(message),
                )));
            }
            0x33 => {
                let reason = std::str::from_utf8(&packet[1..])?;

                let message = CustomLoginFailureMessage {
                    reason: reason.to_owned(),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::CustomLoginFailure(message),
                )));
            }
            0x34 => {
                if packet.len() < 7 {
                    return Err(anyhow!("continuum version message was too small"));
                }

                let message = ContinuumVersionMessage {
                    version: u16::from_le_bytes(packet[1..3].try_into().unwrap()),
                    checksum: u32::from_le_bytes(packet[3..7].try_into().unwrap()),
                };

                return Ok(Some(ServerMessage::Game(
                    GameServerMessage::ContinuumVersion(message),
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
