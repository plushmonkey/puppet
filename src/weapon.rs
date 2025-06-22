#[derive(Debug)]
pub enum WeaponKind {
    None = 0,
    Bullet,
    BouncingBullet,
    Bomb,
    ProximityBomb,
    Repel,
    Decoy,
    Burst,
    Thor,
    Shrapnel = 15,
}

impl Into<WeaponKind> for u16 {
    fn into(self) -> WeaponKind {
        match &self {
            0 => WeaponKind::None,
            1 => WeaponKind::Bullet,
            2 => WeaponKind::BouncingBullet,
            3 => WeaponKind::Bomb,
            4 => WeaponKind::ProximityBomb,
            5 => WeaponKind::Repel,
            6 => WeaponKind::Decoy,
            7 => WeaponKind::Burst,
            8 => WeaponKind::Thor,
            _ => WeaponKind::None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct WeaponData {
    value: u16,
}

impl WeaponData {
    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn kind(&self) -> WeaponKind {
        let kind: WeaponKind = (self.value & 0x1F).into();
        kind
    }

    pub fn level(&self) -> u8 {
        ((self.value >> 5) & 0x03) as u8
    }

    pub fn shrapnel_bouncing(&self) -> bool {
        (self.value >> 7) & 0x01 != 0
    }

    pub fn shrapnel_level(&self) -> u8 {
        ((self.value >> 8) & 0x03) as u8
    }

    pub fn shrapnel_count(&self) -> u8 {
        ((self.value >> 10) & 0x1F) as u8
    }

    pub fn alternate(&self) -> bool {
        (self.value >> 15) & 0x01 != 0
    }
}

impl From<u16> for WeaponData {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for WeaponData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(
            f,
            "WeaponData {{ kind: {:?}, level: {}, shrapnel_bouncing: {}, shrapnel_level: {}, shrapnel_count: {}, alternate: {} }}",
            self.kind(),
            self.level(),
            self.shrapnel_bouncing(),
            self.shrapnel_level(),
            self.shrapnel_count(),
            self.alternate()
        )
    }
}
