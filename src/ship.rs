#[derive(Copy, Clone, Debug)]
pub enum Ship {
    Warbird = 1,
    Javelin,
    Spider,
    Leviathan,
    Terrier,
    Weasel,
    Lancaster,
    Shark,
    Spectator,
}

impl Ship {
    pub fn network_value(&self) -> u8 {
        *self as u8 - 1
    }

    pub fn from_network_value(v: u8) -> Ship {
        match v {
            0 => Ship::Warbird,
            1 => Ship::Javelin,
            2 => Ship::Spider,
            3 => Ship::Leviathan,
            4 => Ship::Terrier,
            5 => Ship::Weasel,
            6 => Ship::Lancaster,
            7 => Ship::Shark,
            8 => Ship::Spectator,
            _ => Ship::Spectator,
        }
    }
}
