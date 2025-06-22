use crate::ship::Ship;
use std::collections::HashMap;

use crate::{
    clock::ServerTick,
    math::{Position, Velocity},
};

#[allow(nonstandard_style)]
pub mod StatusFlags {
    pub const Stealth: u8 = 1 << 0;
    pub const Cloak: u8 = 1 << 1;
    pub const XRadar: u8 = 1 << 2;
    pub const Antiwarp: u8 = 1 << 3;
    pub const Flash: u8 = 1 << 4;
    pub const Safety: u8 = 1 << 5;
    pub const UFO: u8 = 1 << 6;
    pub const Inert: u8 = 1 << 7;
}

#[derive(PartialEq, Clone, Copy, Eq, Hash)]
pub struct PlayerId {
    pub value: u16,
}

impl PlayerId {
    pub fn new(value: u16) -> PlayerId {
        PlayerId { value }
    }

    pub fn invalid() -> PlayerId {
        PlayerId::new(0xFFFF)
    }
}

impl From<u16> for PlayerId {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

pub struct Player {
    pub id: PlayerId,

    pub name: String,
    pub squad: String,

    pub ship: Ship,
    pub frequency: u16,

    pub position: Position,
    pub velocity: Velocity,
    pub direction: u8,

    pub bounty: u16,
    pub status: u8,
    pub ping: u8,

    pub attach_parent: PlayerId,
    pub flag_count: u16,

    pub last_position_timestamp: ServerTick,
}

impl Player {
    pub fn new(id: PlayerId, name: &str, squad: &str, ship: Ship, frequency: u16) -> Self {
        Self {
            id,
            name: name.to_owned(),
            squad: squad.to_owned(),

            position: Position::new(0, 0),
            velocity: Velocity::new(0, 0),
            direction: 0,
            ship,
            frequency,

            bounty: 0,
            status: 0,
            ping: 0,

            attach_parent: PlayerId::invalid(),
            flag_count: 0,

            last_position_timestamp: ServerTick::empty(),
        }
    }
}

pub struct PlayerManager {
    pub players: HashMap<PlayerId, Player>,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn get(&self, id: &PlayerId) -> Option<&Player> {
        self.players.get(id)
    }

    pub fn get_mut(&mut self, id: &PlayerId) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    pub fn add_player(&mut self, player: Player) -> Option<Player> {
        self.players.insert(player.id, player)
    }

    pub fn remove_player(&mut self, id: &PlayerId) -> Option<Player> {
        self.players.remove(id)
    }
}
