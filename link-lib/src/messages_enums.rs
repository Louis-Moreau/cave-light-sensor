use serde::{Serialize,Deserialize};
use crate::Event;
use postcard::experimental::max_size::MaxSize;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Request{
    GetEmbeddedTime,
    SetEmbeddedTime(u64),
    ClearMemory,
    GetNumberofEvent,
    GetEvent(u32),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Response{
    Ok,
    Error,
    Eventinfo(Event),
    CurrentTime(u64),
}