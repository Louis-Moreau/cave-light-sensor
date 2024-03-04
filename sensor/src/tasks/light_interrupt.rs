use rtic::Mutex;
use stm32l0xx_hal::{
    exti::{Exti, ExtiLine, GpioLine},
    gpio::{gpiob::PB0, Floating, Input},
};

use crate::{
    app::exti0_1,
    light_sensor::{self, *},
    storage::event::Event,
    GPIO_LINE,
};

pub fn light_interrupt(mut ctx: exti0_1::Context) {
    if Exti::is_pending(GpioLine::from_raw_line(GPIO_LINE).unwrap()) {
        Exti::unpend(GpioLine::from_raw_line(GPIO_LINE).unwrap());

        let state = ctx.local.light_int_state;
        let timestamp = ctx.shared.rtc.lock(|r| r.now().timestamp());

        let mut light_detected: bool = false;

        let status = state.sensor.read_status().unwrap();
        if status.was_too_high {
            wait_for_dark(&mut state.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
            light_detected = true;

            let event = Event::High(timestamp);
            //WRITE TO EEPROM
        } else if status.was_too_low {
            wait_for_light(&mut state.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
            light_detected = false;

            let event = Event::Low(timestamp);
            //WRITE TO EEPROM
        }

        ctx.shared.speedy.lock(|speedy| {
            *speedy = light_detected;
        });

        //rprintln!("status {:?}", status);
        //let value = ctx.local.sensor.read_lux().unwrap();
        //rprintln!("value {:?}", value as u32);
    }
}

pub struct LightInterruptState {
    pub interrupt_pin: PB0<Input<Floating>>,
    pub sensor: light_sensor::MyOpt3001,
}
