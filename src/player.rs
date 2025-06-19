#![allow(nonstandard_style)]
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
