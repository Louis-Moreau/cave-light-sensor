use eeprom24x::Eeprom24x;

use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::gpio::{OpenDrain, Output};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;


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