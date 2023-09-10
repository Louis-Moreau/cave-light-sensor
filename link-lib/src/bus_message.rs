use heapless::Vec;

pub trait BusMessage<const N : usize> {
    fn is_message_complete(data : &[u8]) -> bool;
    fn decode(data : &mut [u8]) -> Option<Self> where Self: Sized;
    fn decode_request_if_complete(data : &mut [u8]) -> Option<Self> where Self: Sized;
    fn encode_message(&self) -> Result<Vec<u8,N>,()>;
}