#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]


use panic_halt as _;

use rtic::app;
use nb::block;
use rtic_monotonics::systick::*;
use stm32l0xx_hal::gpio::{Output,OpenDrain, PushPull};
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::prelude::*;
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::rcc::Config;
use stm32l0xx_hal::pac::I2C1;
use opt300x::{Opt300x};
use opt300x::ic::Opt3001;
use opt300x::mode::OneShot;

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB3<Output<PushPull>>,
        state: bool,
        sensor : Opt300x<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>, Opt3001, OneShot>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 16_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz

        // Setup LED
        let gpiob = cx.device.GPIOB.split(&mut rcc);

        let sda = gpiob.pb7.into_open_drain_output();
        let scl = gpiob.pb6.into_open_drain_output();
        let i2c = cx.device.I2C1.i2c(sda, scl, embedded_time::rate::units::Hertz(100_000), &mut rcc);
        let sensor = Opt300x::new_opt3001(i2c, opt300x::SlaveAddr::Alternative(false, false));


        let mut led = gpiob
            .pb3
            .into_push_pull_output();
        led.set_high().unwrap();

        // Schedule the blinking task
        blink::spawn().ok();
        read_sensor::spawn().ok();

        (Shared {}, Local { led, state: false , sensor })
    }

    #[task(local = [led, state])]
    async fn blink(cx: blink::Context) {
        loop {
            if *cx.local.state {
                cx.local.led.set_high().unwrap();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low().unwrap();
                *cx.local.state = true;
            }
            Systick::delay(100.millis()).await;
        }
    }

    #[task(local = [sensor])]
    async fn read_sensor(cx: read_sensor::Context) {
        loop {
            let l = block!(cx.local.sensor.read_lux()).unwrap().result;
            let value = l as u32;
            Systick::delay(1000.millis()).await;
        }
    }
}