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

/*fn handle_command(port: TTYPort, command: Commands, logs: Receiver<String>) -> JoinHandle<TTYPort> {
    thread::spawn(||{
        TTYPort::










    })
}*/