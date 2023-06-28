#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_abort as _;
use rtt_target::{rtt_init_print, rprintln};


use rtic::app;
use rtic_monotonics::systick::*;
use stm32l0xx_hal::gpio::gpiob::*;
use stm32l0xx_hal::gpio::{OpenDrain, Output, PushPull};
use stm32l0xx_hal::i2c::I2c;
use stm32l0xx_hal::pac::I2C1;
use stm32l0xx_hal::{
    exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
    gpio::*,
    prelude::*,
    rcc::Config,
    syscfg::SYSCFG,
};

use opt300x::{ic::Opt3001, FaultCount, Opt300x};

const GPIO_LINE: u8 = 0;


const MANTISSA_MAX: u16 = 0xFFF ;
const MANTISSA_MIN: u16 = 0;
const EXPONENT_MAX: u8 = 0b1011 ;
const EXPONENT_MIN: u8 = 0;

const EXPONENT_THRESHOLD : u8 = 0b1001u8;
const MANTISSA_THRESHOLD : u16 = 488u16;


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
        sensor:
        Opt300x<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>, Opt3001, opt300x::mode::Continuous>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 16_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz
        rtt_init_print!();
        // Setup LED
        let gpiob = cx.device.GPIOB.split(&mut rcc);

        let sda = gpiob.pb7.into_open_drain_output();
        let scl = gpiob.pb6.into_open_drain_output();

        let interrupt_pin = gpiob.pb0.into_floating_input();
        let mut syscfg = SYSCFG::new(cx.device.SYSCFG, &mut rcc);
        let mut exti = Exti::new(cx.device.EXTI);

        let i2c = cx.device.I2C1.i2c(
            sda,
            scl,
            embedded_time::rate::units::Hertz(100_000),
            &mut rcc,
        );
        let sensor = Opt300x::new_opt3001(i2c, opt300x::SlaveAddr::Alternative(false, false));
        let mut sensor: Opt300x<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>, Opt3001, opt300x::mode::Continuous> = sensor.into_continuous().ok().unwrap();

        setup_sensor(&mut sensor);

        let linef = GpioLine::from_raw_line(GPIO_LINE).unwrap();
        exti.listen_gpio(
            &mut syscfg,
            interrupt_pin.port(),
            linef,
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

    #[task(binds = EXTI0_1, local = [interrupt_pin,sensor], shared = [speedy])]
    fn exti0_1(mut ctx: exti0_1::Context) {
       if Exti::is_pending(GpioLine::from_raw_line(GPIO_LINE).unwrap()) {
            let mut light : bool = false;

            let status = ctx.local.sensor.read_status().unwrap();
            if status.was_too_high {
                wait_for_dark(ctx.local.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
                light = true;
                rprintln!("HIGH");
            } else if status.was_too_low {
                wait_for_light(ctx.local.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
                light = false;
                rprintln!("LOW");
            }
            
            ctx.shared.speedy.lock(|speedy| {
                *speedy = light;
            });
            Exti::unpend(GpioLine::from_raw_line(GPIO_LINE).unwrap());

            
            //rprintln!("status {:?}", status);
            //let value = ctx.local.sensor.read_lux().unwrap();
            //rprintln!("value {:?}", value as u32);
        }
    }

    fn setup_sensor(
        sensor: &mut OPT3001Sensor
    ) {
        // Set the comparison mode to Lacthed window instead of Hysterersis
        sensor
            .set_comparison_mode(opt300x::ComparisonMode::LatchedWindow)
            .unwrap();

        // We use a pullup resistor so LOW is active
        sensor
            .set_interrupt_pin_polarity(opt300x::InterruptPinPolarity::Low)
            .unwrap();

        // Number of "positive" read that are necessary to activate an interrupt
        sensor.set_fault_count(FaultCount::One).unwrap();

        // exponent = 0b1011 for max range (83k lux) and 20.48lux resolution
        // 488 * 20.48lux = ~10k lux
        // The interrupt is activated at 10k lux and deactivated at 5k lux
        //sensor.set_high_limit_raw(0b1001u8, 488u16).unwrap();
        //sensor.set_low_limit_raw(0b1001u8, 244u16).unwrap();
        sensor.set_integration_time(opt300x::IntegrationTime::Ms800).unwrap();

        wait_for_light(sensor,MANTISSA_THRESHOLD,EXPONENT_THRESHOLD);
    }

    fn wait_for_dark(sensor : &mut OPT3001Sensor,mantissa : u16,exponent : u8) {
        sensor.set_high_limit_raw(EXPONENT_MAX, MANTISSA_MAX).unwrap();
        sensor.set_low_limit_raw(exponent, mantissa).unwrap();
    }

    fn wait_for_light(sensor : &mut OPT3001Sensor,mantissa : u16,exponent : u8) {
        sensor.set_high_limit_raw(exponent, mantissa).unwrap();
        sensor.set_low_limit_raw(EXPONENT_MIN, MANTISSA_MIN).unwrap();
    }

    type OPT3001Sensor = Opt300x<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>, Opt3001, opt300x::mode::Continuous>;
}
