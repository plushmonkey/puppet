use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::From;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(PartialEq, Eq)]
pub struct LocalTick {
    value: u32,
}

impl LocalTick {
    pub fn now() -> Self {
        let tick: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Now must be later than unix epoch")
            .as_millis();
        let tick = (tick / 10) as u32;

        Self::new(tick)
    }

    pub fn new(value: u32) -> Self {
        Self {
            value: value & 0x7FFFFFFF,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn diff(&self, other: &Self) -> i32 {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        (a_value - b_value) >> 1
    }
}

impl PartialOrd for LocalTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        a_value.partial_cmp(&b_value)
    }

    fn lt(&self, other: &Self) -> bool {
        self.diff(other) < 0
    }

    fn le(&self, other: &Self) -> bool {
        self.diff(other) <= 0
    }

    fn gt(&self, other: &Self) -> bool {
        self.diff(other) > 0
    }

    fn ge(&self, other: &Self) -> bool {
        self.diff(other) >= 0
    }
}

impl Ord for LocalTick {
    fn cmp(&self, other: &Self) -> Ordering {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        a_value.cmp(&b_value)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.gt(&other) { self } else { other }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.lt(&other) { self } else { other }
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        let mut result = Self { value: self.value };

        if result.lt(&min) {
            result.value = min.value;
        }

        if result.gt(&max) {
            result.value = max.value;
        }

        result
    }
}

impl From<u32> for LocalTick {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

#[derive(PartialEq, Eq)]
pub struct ServerTick {
    value: u32,
}

impl ServerTick {
    pub fn now() -> Self {
        let tick: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Now must be later than unix epoch")
            .as_millis();
        let tick = (tick / 10) as u32;

        Self::new(tick)
    }

    pub fn new(value: u32) -> Self {
        Self {
            value: value & 0x7FFFFFFF,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn diff(&self, other: &Self) -> i32 {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        (a_value - b_value) >> 1
    }
}

impl PartialOrd for ServerTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        a_value.partial_cmp(&b_value)
    }

    fn lt(&self, other: &Self) -> bool {
        self.diff(other) < 0
    }

    fn le(&self, other: &Self) -> bool {
        self.diff(other) <= 0
    }

    fn gt(&self, other: &Self) -> bool {
        self.diff(other) > 0
    }

    fn ge(&self, other: &Self) -> bool {
        self.diff(other) >= 0
    }
}

impl Ord for ServerTick {
    fn cmp(&self, other: &Self) -> Ordering {
        let a_value = (self.value << 1) as i32;
        let b_value = (other.value << 1) as i32;

        a_value.cmp(&b_value)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.gt(&other) { self } else { other }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.lt(&other) { self } else { other }
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        let mut result = Self { value: self.value };

        if result.lt(&min) {
            result.value = min.value;
        }

        if result.gt(&max) {
            result.value = max.value;
        }

        result
    }
}

impl From<u32> for ServerTick {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}
