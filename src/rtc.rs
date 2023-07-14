#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Status {
    High = 1,
    Low = 2,
    Alive = 3,
}

pub struct Log {
    pub timestamp : u32,
    pub status : Status,
}

impl Log {
    pub fn to_be_bytes(&self) -> [u8;5]  {
        let mut data : [u8;5] = [0u8;5];
        data[..4].copy_from_slice(&self.timestamp.to_be_bytes());
        data[4] = self.status as u8;

        return data
    }
}