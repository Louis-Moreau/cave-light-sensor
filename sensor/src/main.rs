#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_abort as _;

mod eeprom;
mod light_sensor;
mod rtc;
use eeprom::*;
use light_sensor::*;

use eeprom24x::{Eeprom24x, SlaveAddr};
use opt300x::Opt300x;
use rtic::app;
use rtic_monotonics::systick::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32l0xx_hal::gpio::{
    *,
    {gpiob::*, OpenDrain, Output, PushPull},
};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;
use stm32l0xx_hal::{
    exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
    prelude::*,
    pwr::PWR,
    rcc::Config,
    rtc::Rtc,
    syscfg::SYSCFG,
};
use core::fmt::Write;

const GPIO_LINE: u8 = 0;
const ZERO_TIME: u64 = 978307200;

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {

    use super::*;

    #[shared]
    struct Shared {
        speedy: bool,
    }

    #[local]
    struct Local {
        led: PB3<Output<PushPull>>,
        state: bool,
        interrupt_pin: PB0<Input<Floating>>,
        sensor: light_sensor::MyOpt3001,
        eeprom: MyEeprom,
        rtc: Rtc,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        let pwr = PWR::new(cx.device.PWR, &mut rcc);

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 16_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz
        //rtt_init_print!();
        // Setup LED
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioa = cx.device.GPIOA.split(&mut rcc);

        let tx_pin = gpioa.pa1;
        let rx_pin = gpioa.pa3;

        let serial = cx.device
        .LPUART1
        .usart(tx_pin, rx_pin, stm32l0xx_hal::serial::Config::default(), &mut rcc)
        .unwrap();

        let (mut tx, mut rx) = serial.split();

        writeln!(tx, "Hello, world!\n").unwrap();


        let sda = gpiob.pb7.into_open_drain_output();
        let scl = gpiob.pb6.into_open_drain_output();

        let i2c = cx.device.I2C1.i2c(
            sda,
            scl,
            embedded_time::rate::units::Hertz(100_000),
            &mut rcc,
        );

        let shared_i2c  = shared_bus::new_cortexm!(I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>> = i2c).unwrap();
        let eeprom = Eeprom24x::new_24x256(shared_i2c.acquire_i2c(), SlaveAddr::default());

        let sensor = Opt300x::new_opt3001(
            shared_i2c.acquire_i2c(),
            opt300x::SlaveAddr::Alternative(false, false),
        );
        let mut sensor = sensor.into_continuous().ok().unwrap();

        setup_sensor(&mut sensor);

        let rtc = Rtc::new(cx.device.RTC, &mut rcc, &pwr, None).unwrap();

        let interrupt_pin = gpiob.pb0.into_floating_input();
        let mut syscfg = SYSCFG::new(cx.device.SYSCFG, &mut rcc);
        let mut exti = Exti::new(cx.device.EXTI);

        exti.listen_gpio(
            &mut syscfg,
            interrupt_pin.port(),
            GpioLine::from_raw_line(interrupt_pin.pin_number()).unwrap(),
            TriggerEdge::Falling,
        );

        let mut led = gpiob.pb3.into_push_pull_output();
        led.set_high().unwrap();

        // Schedule the blinking task
        blink::spawn().ok();
        let speedy = false;

        (
            Shared { speedy },
            Local {
                interrupt_pin,
                led,
                state: false,
                sensor,
                eeprom,
                rtc,
            },
        )
    }

    #[task(local = [led, state], shared = [speedy])]
    async fn blink(mut ctx: blink::Context) {
        loop {
            if *ctx.local.state {
                ctx.local.led.set_high().unwrap();
                *ctx.local.state = false;
            } else {
                ctx.local.led.set_low().unwrap();
                *ctx.local.state = true;
            }
            let delay: u32 = ctx
                .shared
                .speedy
                .lock(|speedy| if *speedy { 100 } else { 1000 });

            Systick::delay(delay.millis()).await;
        }
    }

    #[task(binds = EXTI0_1, local = [interrupt_pin,sensor,eeprom,rtc], shared = [speedy])]
    fn exti0_1(mut ctx: exti0_1::Context) {
        if Exti::is_pending(GpioLine::from_raw_line(GPIO_LINE).unwrap()) {
            let mut light: bool = false;

            let status = ctx.local.sensor.read_status().unwrap();
            if status.was_too_high {
                wait_for_dark(ctx.local.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
                light = true;
                rprintln!("HIGH");
                increment_stored_count(ctx.local.eeprom);
            } else if status.was_too_low {
                wait_for_light(ctx.local.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
                light = false;
                rprintln!("LOW");
                increment_stored_count(ctx.local.eeprom);
            }
            let now = ctx.local.rtc.now().timestamp() as u64 - ZERO_TIME;
            rprintln!("time : {}", now);

            ctx.shared.speedy.lock(|speedy| {
                *speedy = light;
            });
            Exti::unpend(GpioLine::from_raw_line(GPIO_LINE).unwrap());

            //rprintln!("status {:?}", status);
            //let value = ctx.local.sensor.read_lux().unwrap();
            //rprintln!("value {:?}", value as u32);
        }
    }
}