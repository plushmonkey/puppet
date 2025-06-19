use crate::checksum::weapon_checksum;
use crate::clock::ServerTick;
use crate::net::packet::s2c::ChatKind;
use crate::net::packet::{Packet, Serialize};
use crate::ship::Ship;

// Core packets

pub enum EncryptionClientVersion {
    Subspace,
    ContinuumClassic,
    Continuum,
}

pub struct EncryptionRequestMessage {
    pub key: u32,
    pub version: EncryptionClientVersion,
}

impl EncryptionRequestMessage {
    pub fn new(key: u32) -> Self {
        Self {
            key,
            version: EncryptionClientVersion::Subspace,
        }
    }
}

impl Serialize for EncryptionRequestMessage {
    fn serialize(&self) -> Packet {
        let version = match &self.version {
            EncryptionClientVersion::Subspace => 0x01,
            EncryptionClientVersion::ContinuumClassic => 0x10,
            EncryptionClientVersion::Continuum => 0x11,
        } as u16;

        Packet::empty()
            .concat_u8(0x00)
            .concat_u8(0x01)
            .concat_u32(self.key)
            .concat_u16(version)
    }
}

// Game packets
pub enum ArenaRequest {
    AnyPublic,
    SpecificPublic(u16),
    Name([u8; 16]),
}

// 0x01
pub struct ArenaJoinMessage {
    pub ship: Ship,
    pub resolution_x: u16,
    pub resolution_y: u16,
    pub arena_request: ArenaRequest,
}

impl ArenaJoinMessage {
    pub fn new(
        ship: Ship,
        resolution_x: u16,
        resolution_y: u16,
        arena_request: ArenaRequest,
    ) -> Self {
        Self {
            ship,
            resolution_x,
            resolution_y,
            arena_request,
        }
    }
}

impl Serialize for ArenaJoinMessage {
    fn serialize(&self) -> Packet {
        let mut arena_number = 0xFFFF;
        let mut arena_name = [0; 16];

        match self.arena_request {
            ArenaRequest::AnyPublic => {}
            ArenaRequest::SpecificPublic(number) => {
                arena_number = number;
            }
            ArenaRequest::Name(name) => {
                if name.len() <= 16 {
                    arena_name[..name.len()].copy_from_slice(&name);
                    arena_number = 0xFFFD;
                }
            }
        }

        let ship = self.ship.network_value();

        Packet::empty()
            .concat_u8(0x01)
            .concat_u8(ship)
            .concat_u16(0x01) // Audio
            .concat_u16(self.resolution_x)
            .concat_u16(self.resolution_y)
            .concat_u16(arena_number)
            .concat_bytes(&arena_name)
    }
}

// 0x03
// TODO: Turn these into real types
pub struct PositionMessage {
    pub direction: u8,
    pub timestamp: ServerTick,
    pub x_position: u16,
    pub y_position: u16,
    pub x_velocity: i16,
    pub y_velocity: i16,
    pub togglables: u8,
    pub bounty: u16,
    pub energy: u16,
    pub weapon_info: u16,
}

impl Serialize for PositionMessage {
    fn serialize(&self) -> Packet {
        let mut packet = Packet::empty()
            .concat_u8(0x03)
            .concat_u8(self.direction)
            .concat_u32(self.timestamp.value())
            .concat_i16(self.x_velocity)
            .concat_u16(self.y_position)
            .concat_u8(0x00) // Checksum
            .concat_u8(self.togglables)
            .concat_u16(self.x_position)
            .concat_i16(self.y_velocity)
            .concat_u16(self.bounty)
            .concat_u16(self.energy)
            .concat_u16(self.weapon_info);

        packet.data[10] = weapon_checksum(&packet.data[..packet.size]);

        packet
    }
}

// 0x06
pub struct SendChatMessage<'a> {
    pub kind: ChatKind,
    pub sound: u8,
    pub target_id: u16,
    pub text: &'a str,
}

impl<'a> SendChatMessage<'a> {
    pub fn public(text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::Public,
            sound: 0,
            target_id: 0,
            text,
        }
    }

    // target must be in the same arena
    pub fn private(target_id: u16, text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::Private,
            sound: 0,
            target_id,
            text,
        }
    }

    // text must be formatted as :target:message
    pub fn remote_private(text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::RemotePrivate,
            sound: 0,
            target_id: 0,
            text,
        }
    }

    pub fn team(text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::Team,
            sound: 0,
            target_id: 0,
            text,
        }
    }

    pub fn frequency(frequency: u16, text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::Team,
            sound: 0,
            target_id: frequency,
            text,
        }
    }

    // text must be formatted as 1;message
    pub fn channel(text: &'a str) -> SendChatMessage<'a> {
        SendChatMessage {
            kind: ChatKind::Channel,
            sound: 0,
            target_id: 0,
            text,
        }
    }
}

impl<'a> Serialize for SendChatMessage<'a> {
    fn serialize(&self) -> Packet {
        Packet::empty()
            .concat_u8(0x06)
            .concat_u8(self.kind as u8)
            .concat_u8(self.sound)
            .concat_u16(self.target_id)
            .concat_str(self.text)
    }
}

// 0x09
pub struct PasswordMessage {
    pub new_user: bool,
    pub name: [u8; 32],
    pub password: [u8; 32],
    pub machine_id: u32,
    pub timezone: u16,
    pub version: u16,
    pub permission_id: u32,
}

impl PasswordMessage {
    pub fn new(
        name: &str,
        password: &str,
        new_user: bool,
        machine_id: u32,
        timezone: u16,
        version: u16,
        permission_id: u32,
    ) -> Self {
        let mut new_name = [0; 32];
        new_name[..name.len()].copy_from_slice(name.as_bytes());

        let mut new_password = [0; 32];
        new_password[..password.len()].copy_from_slice(password.as_bytes());

        Self {
            new_user,
            name: new_name,
            password: new_password,
            machine_id,
            timezone,
            version,
            permission_id,
        }
    }
}

impl Serialize for PasswordMessage {
    fn serialize(&self) -> Packet {
        let new_user = if self.new_user { 1 } else { 0 };
        Packet::empty()
            .concat_u8(0x09)
            .concat_u8(new_user)
            .concat_bytes(&self.name)
            .concat_bytes(&self.password)
            .concat_u32(self.machine_id)
            .concat_u8(0x04) // ConnectType
            .concat_u16(self.timezone)
            .concat_u16(0x00)
            .concat_u16(self.version)
            .concat_u32(444)
            .concat_u32(555)
            .concat_u32(self.permission_id)
            .concat_u32(0x00)
            .concat_u32(0x00)
            .concat_u32(0x00)
    }
}

pub struct SecurityMessage {
    pub weapon_count: u32,
    pub settings_checksum: u32,
    pub exe_checksum: u32,
    pub level_checksum: u32,
    pub s2c_slow_total: u32,
    pub s2c_fast_total: u32,
    pub s2c_slow_current: u16,
    pub s2c_fast_current: u16,
    pub s2c_reliable_out: u16,
    pub ping: u16,
    pub ping_average: u16,
    pub ping_low: u16,
    pub ping_high: u16,
    pub slow_frame: bool,
}

impl SecurityMessage {
    pub fn new(
        weapon_count: u32,
        settings_checksum: u32,
        exe_checksum: u32,
        level_checksum: u32,
    ) -> SecurityMessage {
        SecurityMessage {
            weapon_count,
            settings_checksum,
            exe_checksum,
            level_checksum,
            s2c_slow_total: 0,
            s2c_fast_total: 0,
            s2c_slow_current: 0,
            s2c_fast_current: 0,
            s2c_reliable_out: 0,
            ping: 0,
            ping_average: 0,
            ping_low: 0,
            ping_high: 0,
            slow_frame: false,
        }
    }
}

impl Serialize for SecurityMessage {
    fn serialize(&self) -> Packet {
        let slow_frame = if self.slow_frame { 1 } else { 0 };

        Packet::empty()
            .concat_u8(0x1A)
            .concat_u32(self.weapon_count)
            .concat_u32(self.settings_checksum)
            .concat_u32(self.exe_checksum)
            .concat_u32(self.level_checksum)
            .concat_u32(self.s2c_slow_total)
            .concat_u32(self.s2c_fast_total)
            .concat_u16(self.s2c_slow_current)
            .concat_u16(self.s2c_fast_current)
            .concat_u16(self.s2c_reliable_out)
            .concat_u16(self.ping)
            .concat_u16(self.ping_average)
            .concat_u16(self.ping_low)
            .concat_u16(self.ping_high)
            .concat_u8(slow_frame)
    }
}
