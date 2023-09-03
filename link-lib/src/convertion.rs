
use heapless::Vec;
use postcard::experimental::max_size::MaxSize;
use crate::{Response, Request};


const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}
const COBS_OVERHEAD_SIZE : usize = 2;
pub const MAX_OBJECT_SIZE: usize = max(Response::POSTCARD_MAX_SIZE,Request::POSTCARD_MAX_SIZE) + COBS_OVERHEAD_SIZE;

pub fn is_object_complete(data : &mut [u8]) -> bool {
    if let Some(b) = data.last() {
        return *b == 0x00u8;
    } else {
        return false;
    }
}

pub fn decode_response(data : &mut [u8]) -> Option<Response> {
    match postcard::from_bytes_cobs(data) {
        Ok(o) => Some(o),
        Err(_) => None,
    } 
}

pub fn decode_response_if_complete(data : &mut [u8]) -> Option<Response> {
    if is_object_complete(data) {
        return decode_response(data);
    } else {
        return None;
    }
}

pub fn decode_request(data : &mut [u8]) -> Option<Request> {
    match postcard::from_bytes_cobs(data) {
        Ok(o) => Some(o),
        Err(_) => None,
    } 
}

pub fn decode_request_if_complete(data : &mut [u8]) -> Option<Request> {
    if is_object_complete(data) {
        return decode_request(data);
    } else {
        return None;
    }
}

pub fn encode_data<D : serde::Serialize>(object : D) -> Result<Vec<u8,MAX_OBJECT_SIZE>,()> {
    /*  Does not work for the moment since CrcModifier does not implement trait IndexMut and Cobs needs it
        let res = serialize_with_flavor::<Request, Cobs<CrcModifier<HVec<MAX_OBJECT_STRING_SIZE>,u16>>, &mut Vec<u8,MAX_OBJECT_STRING_SIZE>>
        (object,CrcModifier::new(HVec::default(),digest)).unwrap();
    */

    match postcard::to_vec_cobs(&object) {
        Ok(o) => Ok(o),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Event, EventType};

    use super::*;

    #[test]
    fn encoding_decoding_response_success() {
        let response = Response::Eventinfo( Event { timestamp: 102654646, event_type: EventType::High });
        let mut encoded = encode_data(response).unwrap();
        let converted_back = decode_response_if_complete(&mut encoded).unwrap();
        assert_eq!(response, converted_back);
    }

    #[test]
    fn encoding_decoding_response_fail() {
        let response = Response::Eventinfo( Event { timestamp: 102654646, event_type: EventType::High });
        let mut encoded = encode_data(response).unwrap();
        let last = encoded.last_mut().unwrap();
        *last += 1;
        let converted_back = decode_response_if_complete(&mut encoded);
        assert_eq!(converted_back, None);
    }

    #[test]
    fn size_test_response() {
        let response = Response::Eventinfo( Event { timestamp: u32::MAX, event_type: EventType::Alive });
        let encoded = encode_data(response).unwrap();
        assert!(encoded.len() < MAX_OBJECT_SIZE)
    }

    #[test]
    fn size_test_request() {
        assert!(Request::POSTCARD_MAX_SIZE <= MAX_OBJECT_SIZE)
    }
}



