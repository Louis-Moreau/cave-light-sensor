use core::marker::PhantomData;

use heapless::Vec;
use postcard::{accumulator::CobsAccumulator, experimental::max_size::MaxSize};
use crate::error::MyError;


pub struct Link<S,REQ,RESP,const SIZE : usize> {
    serial: S,
    buffer : CobsAccumulator<SIZE>,
    phantom_req: PhantomData<REQ>,
    phantom_resp: PhantomData<RESP>
}

impl<'a, S,REQ,RESP,const SIZE : usize> Link<S,REQ,RESP,SIZE>
where
    S: embedded_hal::serial::Read<u8> + embedded_hal::blocking::serial::Write<u8>,
    REQ :  for<'de> serde::Deserialize<'de> + MaxSize  + Clone,
    RESP : serde::Serialize
{
    pub fn new(serial: S) -> Self {
        Self {
            serial : serial,
            buffer : CobsAccumulator::new(),
            phantom_req : PhantomData,
            phantom_resp: PhantomData
        }
    }

    pub fn send_response(&mut self,response : &RESP) -> Result<(), MyError<<S as embedded_hal::blocking::serial::Write<u8>>::Error>> {
        let vec: Vec<u8, SIZE> = postcard::to_vec_cobs(&response).map_err(|_|MyError::Serialize)?;

        self.serial.bwrite_all(&vec).map_err(|e|MyError::IO(e))?;
        self.serial.bflush().map_err(|e|MyError::IO(e))?;
        Ok(())
    }

    pub fn get_request(&mut self) -> Result<Option<REQ>, MyError<<S as embedded_hal::serial::Read<u8>>::Error>> {
        let byte = nb::block!(self.serial.read()).map_err(|e|MyError::IO(e))?;

       match self.buffer.feed::<REQ>(&[byte]) {
            postcard::accumulator::FeedResult::Consumed => Ok(None),
            postcard::accumulator::FeedResult::OverFull(_) => Err(MyError::BufferFull),
            postcard::accumulator::FeedResult::DeserError(_) => Err(MyError::Deserialize),
            postcard::accumulator::FeedResult::Success { data, remaining: _ } => Ok(Some(data)),
        }
    }
}
