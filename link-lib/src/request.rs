use serde::{Serialize,Deserialize};
use postcard::experimental::max_size::MaxSize;
use heapless::Vec;
use crate::bus_message::BusMessage;

pub const MAX_REQUEST_SIZE: usize = Request::POSTCARD_MAX_SIZE + 2;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Request{
    GetEmbeddedTime,
    SetEmbeddedTime(u64),
    ClearMemory,
    GetNumberofEvent,
    GetEvent(u32),
}

impl BusMessage<MAX_REQUEST_SIZE> for Request {
    fn is_message_complete(data : &[u8]) -> bool {
        if let Some(b) = data.last() {
            return *b == 0x00u8;
        } else {
            return false;
        }
    }

    fn decode(data : &mut [u8]) -> Option<Self> {
        match postcard::from_bytes_cobs(data) {
            Ok(o) => Some(o),
            Err(_) => None,
        } 
    }

    fn decode_request_if_complete(data : &mut [u8]) -> Option<Self> {
        if Self::is_message_complete(data) {
            return Self::decode(data);
        } else {
            return None;
        }
    }

    fn encode_message(&self) -> Result<Vec<u8,MAX_REQUEST_SIZE>,()> {
        match postcard::to_vec_cobs(self) {
            Ok(o) => Ok(o),
            Err(_) => Err(()),
        }
    }
} 