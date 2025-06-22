pub struct VieRng {
    pub seed: i32,
}

impl VieRng {
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    pub fn next(&mut self) -> u32 {
        let mut seed = self.seed;

        let start = (seed % 0x1F31D).wrapping_mul(0x41A7);
        let mid = (seed / 0x1F31D).wrapping_mul(0xB14);
        let end = 0x7B;

        seed = start.wrapping_sub(mid).wrapping_add(end);
        if seed < 1 {
            seed += 0x7FFFFFFF;
        }

        self.seed = seed;

        return seed as u32;
    }

    pub fn next_encrypt(&mut self) -> u16 {
        let old_seed = self.seed;
        let mut seed = ((old_seed as u64).wrapping_mul(0x834E0B5F) >> 48) as i32;
        seed = seed.wrapping_add(seed >> 31);

        let start = (old_seed % 0x1F31D).wrapping_mul(0x41A7);
        let mid = seed.wrapping_mul(0xB14);
        let end = 0x7B;

        seed = start.wrapping_sub(mid).wrapping_add(end);
        if seed < 1 {
            seed += 0x7FFFFFFF;
        }

        self.seed = seed;

        seed as u16
    }
}
