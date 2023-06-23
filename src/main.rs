#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_abort as _;

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
            TriggerEdge::Both,
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

    #[task(binds = EXTI0_1, local = [interrupt_pin], shared = [speedy])]
    fn exti0_1(mut ctx: exti0_1::Context) {
       if Exti::is_pending(GpioLine::from_raw_line(GPIO_LINE).unwrap()) {
            let state = ctx.local.interrupt_pin.is_low().unwrap();
            ctx.shared.speedy.lock(|speedy| {
                *speedy = state;
            });
            Exti::unpend(GpioLine::from_raw_line(GPIO_LINE).unwrap());
        }
    }

    fn setup_sensor(
        sensor: &mut Opt300x<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>, Opt3001, opt300x::mode::Continuous>
    ) {
        // Set the comparison mode to Lacthed window instead of Hysterersis
        sensor
            .set_comparison_mode(opt300x::ComparisonMode::TransparentHysteresis)
            .unwrap();

        // We use a pullup resistor so LOW is active
        sensor
            .set_interrupt_pin_polarity(opt300x::InterruptPinPolarity::Low)
            .unwrap();

        // Number of "positive" read that are necessary to activate an interrupt
        sensor.set_fault_count(FaultCount::Four).unwrap();

        // exponent = 0b1011 for max range (83k lux) and 20.48lux resolution
        // 488 * 20.48lux = ~10k lux
        // The interrupt is activated at 10k lux and deactivated at 5k lux
        sensor.set_high_limit_raw(0b1001u8, 488u16).unwrap();
        sensor.set_low_limit_raw(0b1001u8, 244u16).unwrap();
    }
}
