pub enum Ship {
    Warbird,
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
    pub fn value(&self) -> u8 {
        match self {
            Ship::Warbird => 0,
            Ship::Javelin => 1,
            Ship::Spider => 2,
            Ship::Leviathan => 3,
            Ship::Terrier => 4,
            Ship::Weasel => 5,
            Ship::Lancaster => 6,
            Ship::Shark => 7,
            Ship::Spectator => 8,
        }
    }
}
