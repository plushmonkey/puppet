pub type TileId = u8;

pub const TILE_ID_FIRST_DOOR: TileId = 162;
pub const TILE_ID_LAST_DOOR: TileId = 169;
pub const TILE_ID_FLAG: TileId = 170;
pub const TILE_ID_SAFE: TileId = 171;
pub const TILE_ID_GOAL: TileId = 172;
pub const TILE_ID_WORMHOLE: TileId = 220;

pub struct Map {
    pub checksum: u32,
    pub filename: String,
    pub tiles: Box<[TileId; 1024 * 1024]>,
}

impl Map {
    pub fn new(checksum: u32, filename: &str, data: &[u8]) -> Option<Map> {
        let mut map = Map {
            checksum,
            filename: filename.to_owned(),
            tiles: vec![0; 1024 * 1024].into_boxed_slice().try_into().unwrap(),
        };

        let mut position: usize = 0;

        if data.len() >= 4 {
            // If we have a bitmap header, jump to tile data by reading header.
            if data[0] == b'B' && data[1] == b'M' {
                position = u16::from_le_bytes(data[2..4].try_into().unwrap()) as usize;
            }
        }

        if position >= data.len() {
            return None;
        }

        let tile_count = (data.len() - position) / size_of::<u32>();

        for _ in 0..tile_count {
            let tile = u32::from_le_bytes(data[position..position + 4].try_into().unwrap());

            let x = (tile >> 0) & 0xFFF;
            let y = (tile >> 12) & 0xFFF;
            let tile_id = ((tile >> 24) & 0xFF) as u8;

            let index = y as usize * 1024 + x as usize;
            map.tiles[index] = tile_id;

            position += 4;
        }

        Some(map)
    }

    pub fn empty(checksum: u32, filename: &str) -> Map {
        Map {
            checksum,
            filename: filename.to_owned(),
            tiles: vec![0; 1024 * 1024].into_boxed_slice().try_into().unwrap(),
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileId {
        let index = Map::get_index(x, y);

        self.tiles[index]
    }

    pub fn is_door(&self, x: u16, y: u16) -> bool {
        let tile_id = self.get_tile(x, y);
        tile_id >= TILE_ID_FIRST_DOOR && tile_id <= TILE_ID_LAST_DOOR
    }

    pub fn is_solid(&self, x: u16, y: u16) -> bool {
        let tile_id = self.get_tile(x, y);

        if tile_id == 0 {
            return false;
        }

        if tile_id < 170 {
            return true;
        }

        if tile_id == 220 {
            return false;
        }

        if tile_id >= 192 && tile_id <= 240 {
            return true;
        }

        if tile_id >= 242 && tile_id <= 252 {
            return true;
        }

        false
    }

    pub fn is_solid_empty_doors(&self, x: u16, y: u16) -> bool {
        let tile_id = self.get_tile(x, y);

        if tile_id == 0 {
            return false;
        }

        if tile_id >= TILE_ID_FIRST_DOOR && tile_id <= TILE_ID_LAST_DOOR {
            return false;
        }

        if tile_id < 170 {
            return true;
        }

        if tile_id == 220 {
            return false;
        }

        if tile_id >= 192 && tile_id <= 240 {
            return true;
        }

        if tile_id >= 242 && tile_id <= 252 {
            return true;
        }

        false
    }

    fn get_index(x: u16, y: u16) -> usize {
        y as usize * 1024 + x as usize
    }
}
