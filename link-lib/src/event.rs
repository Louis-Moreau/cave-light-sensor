use serde::{Serialize,Deserialize};
use postcard::experimental::max_size::MaxSize;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub struct Event {
    pub timestamp: u32,
    pub event_type: EventType,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_convertion() {
        let event = Event { timestamp: 102654646u32, event_type: EventType::High };
        let mut bytes = event.to_be_bytes();
        let converted = Event::from_be_bytes(&bytes);
        assert_eq!(event, converted);
        bytes[3] += 1;
        let converted = Event::from_be_bytes(&bytes);
        assert_ne!(event, converted);
    }
}
