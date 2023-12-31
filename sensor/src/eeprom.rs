use eeprom24x::Eeprom24x;
use link_lib::Event;
use nb::block;
use postcard::experimental::max_size::MaxSize;
use rtt_target::rprintln;
use stm32l0xx_hal::{
    gpio::{gpiob::*, OpenDrain, Output},
    i2c::I2c,
    pac::I2C1,
};

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

pub fn write_event_to_eeprom(eeprom: &mut MyEeprom, event: Event) ->  Result<(),()> {
    let count = read_count_from_eeprom(eeprom)?;
    let event_data = match event.to_bytes() {
        Ok(e) => e,
        Err(_) => return Err(()),
    };
    write_data_to_eeprom_blocking(eeprom, get_event_adress(count), &event_data)?;
    increment_stored_count(eeprom)?;
    Ok(())
}

pub fn read_event_from_eeprom(eeprom: &mut MyEeprom, number: u32) -> Result<Event,()> {
    let mut data = [0u8; EVENT_SIZE as usize];
    eeprom
        .read_data(get_event_adress(number), &mut data)
        .map_err(|_|())?;

    match Event::from_bytes(&data) {
        Ok(e) => Ok(e),
        Err(_) => Ok(Event::Err),
    }
}

pub fn increment_stored_count(eeprom: &mut MyEeprom) ->  Result<(),()> {
    let mut count = read_count_from_eeprom(eeprom)?;
    count += 1;
    rprintln!("incremented count : {}", count);
    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_le_bytes())?;
    Ok(())
}

pub fn read_count_from_eeprom(eeprom: &mut MyEeprom) -> Result<u32,()> {
    let mut data = [0u8; COUNT_SIZE as usize];
    eeprom.read_data(COUNT_ADDRESS, &mut data).map_err(|_|())?;
    Ok(u32::from_le_bytes(data))
}

pub fn write_data_to_eeprom_blocking(eeprom: &mut MyEeprom, address: u32, data: &[u8]) ->  Result<(),()> {
    for (offset, byte) in data.iter().enumerate() {
        match write_byte_to_eeprom_blocking(eeprom, address + offset as u32, *byte) {
            Ok(_) => (),
            Err(_) => return Err(()),
        }
    }
    return Ok(())
}

pub fn write_byte_to_eeprom_blocking(eeprom: &mut MyEeprom, address: u32, byte: u8) -> Result<(),()>{
    block!(match eeprom.write_byte(address, byte) {
        Ok(_) => Ok(()),
        Err(e) => match e {
            eeprom24x::Error::I2C(_) => Err(nb::Error::WouldBlock),
            eeprom24x::Error::TooMuchData => Err(nb::Error::Other(())),
            eeprom24x::Error::InvalidAddr => Err(nb::Error::Other(())),
        },
    })
}

pub fn zero_stored_count(eeprom: &mut MyEeprom) -> Result<(),()> {
    let count: u32 = 0;
    write_data_to_eeprom_blocking(eeprom, COUNT_ADDRESS, &count.to_le_bytes()).map_err(|_|())?;
    Ok(())
}

pub fn write_sensor_id(eeprom: &mut MyEeprom, sensor_id: u64) ->  Result<(),()> {
    write_data_to_eeprom_blocking(eeprom, SENSOR_ID_ADDRESS, &sensor_id.to_le_bytes())
}

pub fn read_sensor_id(eeprom: &mut MyEeprom) -> Result<u64,()> {
    let mut data = [0u8; SENSOR_ID_SIZE as usize];
    eeprom.read_data(SENSOR_ID_ADDRESS, &mut data).map_err(|_|())?;
    Ok(u64::from_le_bytes(data))
}

fn get_event_adress(count: u32) -> u32 {
    EVENT_ADDRESS + (EVENT_SIZE as u32 * count)
}
