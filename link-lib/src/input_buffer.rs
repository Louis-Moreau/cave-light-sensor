use heapless::Vec;

use crate::{response::Response, bus_message::BusMessage, request::Request};

pub struct RequestBuffer<const N : usize>{
    buffer : Vec<u8,N>
}

impl<const N : usize> RequestBuffer<N> {
    pub fn add_byte(&mut self,byte : &u8) -> Result<(),()> {
        match self.buffer.push(byte.clone()) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn add_byte_and_check_for_response(&mut self,byte : &u8) -> Result<Option<Response>,()> {
        self.add_byte(byte)?;
    
        match Response::decode_request_if_complete(&mut self.buffer) {
            Some(o) => Ok(Some(o)),
            None => Ok(None),
        }
    }

    pub fn add_byte_and_check_for_request(&mut self,byte : &u8) -> Result<Option<Request>,()> {
        self.add_byte(byte)?;

        match Request::decode_request_if_complete(&mut self.buffer) {
            Some(o) => Ok(Some(o)),
            None => Ok(None),
        }
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }


}