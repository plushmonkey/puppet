pub struct Map {
    pub checksum: u32,
    pub data: Vec<u8>,
    pub filename: String,
}

impl Map {
    pub fn new(checksum: u32, filename: &str, data: &[u8]) -> Map {
        Map {
            checksum,
            data: data.to_vec(),
            filename: filename.to_owned(),
        }
    }

    pub fn empty(checksum: u32, filename: &str) -> Map {
        Map {
            checksum,
            data: vec![],
            filename: filename.to_owned(),
        }
    }
}
