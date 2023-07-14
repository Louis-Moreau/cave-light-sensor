use eeprom24x::Eeprom24x;

use rtt_target::rprintln;
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::gpio::{OpenDrain, Output};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;
use nb::block;

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

const COUNT_ADDRESS : u32 = 0x0001;

pub fn increment_stored_count(eeprom : &mut MyEeprom) {
    let mut data: [u8;4] = [0u8;4];
    eeprom.read_data(COUNT_ADDRESS, &mut data).unwrap();
    let mut count: u32 = u32::from_be_bytes(data);
    
    count += 1;
    rprintln!("incremented count : {}",count);
    data = count.to_be_bytes();

    for (offset,byte) in data.iter().enumerate() {
        write_to_eeprom_blocking(eeprom,COUNT_ADDRESS + offset as u32,*byte);
    }

}

pub fn write_to_eeprom_blocking(eeprom : &mut MyEeprom,address : u32,byte : u8) {
    block!(match eeprom.write_byte(address, byte) {
        Ok(_) => Ok(()),
        Err(e) => match e {
            eeprom24x::Error::I2C(_) => Err(nb::Error::WouldBlock),
            eeprom24x::Error::TooMuchData => Err(nb::Error::Other(())),
            eeprom24x::Error::InvalidAddr => Err(nb::Error::Other(())),
        },
    }).unwrap()
}