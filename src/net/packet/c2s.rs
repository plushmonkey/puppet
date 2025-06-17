use crate::net::packet::{Packet, Serialize};
use crate::ship::Ship;

pub enum EncryptionClientVersion {
    Subspace,
    ContinuumClassic,
    Continuum,
}

pub struct EncryptionRequestPacket {
    pub key: u32,
    pub version: EncryptionClientVersion,
}

impl EncryptionRequestPacket {
    pub fn new(key: u32) -> Self {
        Self {
            key,
            version: EncryptionClientVersion::Subspace,
        }
    }
}

impl Serialize for EncryptionRequestPacket {
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

pub struct PasswordPacket {
    pub new_user: bool,
    pub name: [u8; 32],
    pub password: [u8; 32],
    pub machine_id: u32,
    pub timezone: u16,
    pub version: u16,
    pub permission_id: u32,
}

impl PasswordPacket {
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

impl Serialize for PasswordPacket {
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

pub enum ArenaRequest {
    AnyPublic,
    SpecificPublic(u16),
    Name([u8; 16]),
}

pub struct ArenaJoinPacket {
    pub ship: Ship,
    pub resolution_x: u16,
    pub resolution_y: u16,
    pub arena_request: ArenaRequest,
}

impl ArenaJoinPacket {
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

impl Serialize for ArenaJoinPacket {
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

        Packet::empty()
            .concat_u8(0x01)
            .concat_u8(self.ship.value())
            .concat_u16(0x01) // Audio
            .concat_u16(self.resolution_x)
            .concat_u16(self.resolution_y)
            .concat_u16(arena_number)
            .concat_bytes(&arena_name)
    }
}
