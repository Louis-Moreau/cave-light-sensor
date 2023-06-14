#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_rtt_target as _;
use rtic::app;
use rtic_monotonics::systick::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32l0xx_hal::gpio::{Output, PushPull};
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::prelude::*;
use stm32l0xx_hal::rcc::Config;

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PB3<Output<PushPull>>,
        state: bool,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 16_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz

        rtt_init_print!();
        rprintln!("init");

        // Setup LED
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let mut led = gpiob
            .pb3
            .into_push_pull_output();
        led.set_high().unwrap();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led, state: false })
    }

    #[task(local = [led, state])]
    async fn blink(cx: blink::Context) {
        loop {
            rprintln!("blink");
            if *cx.local.state {
                cx.local.led.set_high().unwrap();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low().unwrap();
                *cx.local.state = true;
            }
            Systick::delay(1000.millis()).await;
        }
    }
}