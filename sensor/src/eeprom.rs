use chrono::offset;
use eeprom24x::Eeprom24x;

use nb::block;
use rtt_target::rprintln;
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::gpio::{OpenDrain, Output};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;

use link_lib::{Event, EventType};

pub type MyEeprom = Eeprom24x<
    shared_bus::I2cProxy<
        'static,
        cortex_m::interrupt::Mutex<
            core::cell::RefCell<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>>,
        >,
    >,
    eeprom24x::page_size::B64,
    eeprom24x::addr_size::TwoBytes,
>;

const COUNT_ADDRESS: u32 = 0x0000;
const COUNT_OFFSET: u32 = 0x0004;
const EVENT_SIZE: usize = 0x0005;

pub fn write_event_to_eeprom(eeprom: &mut MyEeprom, event: Event) {
    let mut count = read_count_from_eeprom(eeprom);
    let data: [u8; EVENT_SIZE] = event.to_be_bytes();
    write_data_to_eeprom_blocking(eeprom, COUNT_OFFSET + (EVENT_SIZE as u32 * count), &data);
    count += 1;
    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_be_bytes());
}

pub fn read_event_from_eeprom(eeprom: &mut MyEeprom, number: u32) -> Event {
    let mut data: [u8; EVENT_SIZE] = [0u8; EVENT_SIZE];
    eeprom
        .read_data(COUNT_OFFSET + (EVENT_SIZE as u32 * number), &mut data)
        .unwrap();

    Event::from_be_bytes(&data)
}

pub fn increment_stored_count(eeprom: &mut MyEeprom) {
    let mut count = read_count_from_eeprom(eeprom);
    count += 1;
    rprintln!("incremented count : {}", count);

    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_be_bytes());
}

pub fn read_count_from_eeprom(eeprom: &mut MyEeprom) -> u32 {
    let mut data: [u8; 4] = [0u8; 4];
    eeprom.read_data(COUNT_ADDRESS, &mut data).unwrap();
    u32::from_be_bytes(data)
}

pub fn write_data_to_eeprom_blocking(eeprom: &mut MyEeprom, address: u32, data: &[u8]) {
    for (offset, byte) in data.iter().enumerate() {
        write_byte_to_eeprom_blocking(eeprom, address + offset as u32, *byte);
    }
}

pub fn write_byte_to_eeprom_blocking(eeprom: &mut MyEeprom, address: u32, byte: u8) {
    block!(match eeprom.write_byte(address, byte) {
        Ok(_) => Ok(()),
        Err(e) => match e {
            eeprom24x::Error::I2C(_) => Err(nb::Error::WouldBlock),
            eeprom24x::Error::TooMuchData => Err(nb::Error::Other(())),
            eeprom24x::Error::InvalidAddr => Err(nb::Error::Other(())),
        },
    })
    .unwrap()
}
