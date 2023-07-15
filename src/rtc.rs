#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EventType {
    High = 1,
    Low = 2,
    Alive = 3,
}

impl TryFrom<u8> for EventType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::High),
            2 => Ok(Self::Low),
            3 => Ok(Self::Alive),
            _ => Err(()),
        }
    }
}

pub struct Event {
    pub timestamp: u32,
    pub event_type: EventType,
}

impl Event {
    pub fn to_be_bytes(&self) -> [u8; 5] {
        let mut data: [u8; 5] = [0u8; 5];
        data[..4].copy_from_slice(&self.timestamp.to_be_bytes());
        data[4] = self.event_type as u8;

        return data;
    }

    pub fn from_be_bytes(data: &[u8; 5]) -> Self {
        Self {
            timestamp: u32::from_be_bytes(data[0..4].try_into().unwrap()),
            event_type: EventType::try_from(data[4]).unwrap(),
        }
    }
}
