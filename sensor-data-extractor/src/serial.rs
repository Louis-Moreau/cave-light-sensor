use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Result};
use link_lib::MessageBuffer;
use link_lib::{BusMessage, Request, Response};
use serialport::SerialPort;
use serialport::TTYPort;
use serialport::{new, SerialPortInfo};

use crate::commands::Commands;

fn serial_handler(port: TTYPort, commands_receiver: Receiver<Commands>, logs: Sender<String>) {}

struct SerialHandler {
    thread: Option<JoinHandle<()>>,
    tx_disconnect: Option<Sender<bool>>,
    tx_requests: Option<Sender<Request>>,
    rx_responses: Option<Receiver<Response>>,
}

impl SerialHandler {
    pub fn new() -> Self {
        Self {
            thread: None,
            tx_disconnect: None,
            tx_requests: None,
            rx_responses: None,
        }
    }

    pub fn connect(&mut self, info: SerialPortInfo) -> Result<(), ()> {
        self.thread = Some(thread::spawn(|| {
            // some work here
        }));
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        if let Some(t) = self.thread {
            return !t.is_finished()
                && self.tx_disconnect.is_some()
                && self.tx_requests.is_some()
                && self.rx_responses.is_some();
        }
        return false;
    }

    pub fn get_next_response(&mut self) -> Result<Option<Response>, ()> {
        if let Some(rx) = self.rx_responses {
            match rx.try_recv() {
                Ok(res) => Ok(Some(res)),
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => Ok(None),
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        self.rx_responses = None;
                        Err(())
                    }
                },
            }
        } else {
            return Err(());
        }
    }

    pub fn send_request(&mut self, request: Request) -> Result<(), ()> {
        if let Some(tx) = self.tx_requests {
            match tx.send(request) {
                Ok(_) => Ok(()),
                Err(e) => {
                    self.tx_requests = None;
                    Err(())
                }
            }
        } else {
            return Err(());
        }
    }
}
/*fn get_time_command(port : &TTYPort) -> String {

}*/

fn get_response(port: &mut TTYPort) -> Result<Response> {
    let mut buffer = MessageBuffer::<20>::new();
    let mut response: Option<Response> = None;
    while response.is_none() {
        let mut byte: [u8; 1] = [0u8];
        port.read_exact(byte.as_mut_slice())?;
        response = buffer
            .add_byte_and_check_for_response(&byte[0])
            .map_err(|_| anyhow!("Could not add byte"))?;
    }
    Ok(response.unwrap())
}

fn send_request(port: &mut TTYPort, req: Request) -> Result<()> {
    let encoded_message = req
        .encode_message()
        .map_err(|_| anyhow!("Could not encode request"))?;
    port.write_all(&encoded_message)
        .map_err(|_| anyhow!("Could not send the request"))?;
    Ok(())
}
