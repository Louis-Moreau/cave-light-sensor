use eeprom24x::Eeprom24x;

use nb::block;
use postcard::experimental::max_size::MaxSize;
use rtt_target::rprintln;
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::gpio::{OpenDrain, Output};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;

use link_lib::Event;

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

const COUNT_ADDRESS: u32 = 0;
const COUNT_SIZE: u32 = 4;
const SENSOR_ID_ADDRESS: u32 = COUNT_SIZE;
const SENSOR_ID_SIZE: u32 = 8;
const EVENT_ADDRESS: u32 = SENSOR_ID_ADDRESS + SENSOR_ID_SIZE;
const EVENT_SIZE: u32 = Event::POSTCARD_MAX_SIZE as u32;

pub fn write_event_to_eeprom(eeprom: &mut MyEeprom, event: Event) {
    let count = read_count_from_eeprom(eeprom);
    let event_data = match event.to_bytes() {
        Ok(e) => e,
        Err(_) => return,
    };
    write_data_to_eeprom_blocking(eeprom, get_event_adress(count), &event_data);
    increment_stored_count(eeprom);
}

pub fn read_event_from_eeprom(eeprom: &mut MyEeprom, number: u32) -> Event {
    let mut data = [0u8; EVENT_SIZE as usize];
    eeprom
        .read_data(get_event_adress(number), &mut data)
        .unwrap();

    match Event::from_bytes(&data) {
        Ok(e) => e,
        Err(_) => Event::Err,
    }
}

pub fn increment_stored_count(eeprom: &mut MyEeprom) {
    let mut count = read_count_from_eeprom(eeprom);
    count += 1;
    rprintln!("incremented count : {}", count);
    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_le_bytes());
}

pub fn read_count_from_eeprom(eeprom: &mut MyEeprom) -> u32 {
    let mut data = [0u8; COUNT_SIZE as usize];
    eeprom.read_data(COUNT_ADDRESS, &mut data).unwrap();
    u32::from_le_bytes(data)
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

pub fn zero_stored_count(eeprom: &mut MyEeprom) {
    let count: u32 = 0;
    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_le_bytes());
}

pub fn write_sensor_id(eeprom: &mut MyEeprom, sensor_id: u64) {
    write_data_to_eeprom_blocking(eeprom, SENSOR_ID_ADDRESS, &sensor_id.to_le_bytes());
}

pub fn read_sensor_id(eeprom: &mut MyEeprom) -> u64 {
    let mut data = [0u8; SENSOR_ID_SIZE as usize];
    eeprom.read_data(SENSOR_ID_ADDRESS, &mut data).unwrap();
    u64::from_le_bytes(data)
}

fn get_event_adress(count: u32) -> u32 {
    EVENT_ADDRESS + (EVENT_SIZE as u32 * count)
}
