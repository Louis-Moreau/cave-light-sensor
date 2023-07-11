#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use embedded_sdmmc::TimeSource;
use embedded_sdmmc::Timestamp;
use panic_reset as _;

use rtic::app;
use stm32l0xx_hal::{delay::Delay, gpio::*, prelude::*, rcc::Config};

use embedded_sdmmc::VolumeManager;

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let _ = test_main(cx);

        panic!();
    }

    fn test_main(cx: init::Context) -> Result<(),()> {
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        //let systick_mono_token = rtic_monotonics::create_systick_token!();

        //Systick::start(cx.core.SYST, 16_000_000, systick_mono_token);
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioa = cx.device.GPIOA.split(&mut rcc);

        let nss = gpioa.pa11.into_push_pull_output();
        let sck = gpiob.pb3;
        let miso = gpiob.pb4;
        let mosi = gpiob.pb5;

        // Initialise the SPI peripheral.
        let sdmmc_spi = cx.device.SPI1.spi(
            (sck, miso, mosi),
            stm32l0xx_hal::spi::MODE_0,
            100_000.Hz(),
            &mut rcc,
        );

        let delay = Delay::new(cx.core.SYST, rcc.clocks);

        let sdcard = embedded_sdmmc::SdCard::new(sdmmc_spi, nss, delay);

        let mut volume_mgr = VolumeManager::new(sdcard, Clock);
        let mut volume0 = volume_mgr.get_volume(embedded_sdmmc::VolumeIdx(0)).map_err(drop)?;
        let root_dir = volume_mgr.open_root_dir(&volume0).map_err(drop)?;
        let mut my_file = volume_mgr
            .open_file_in_dir(
                &mut volume0,
                &root_dir,
                "m.TXT",
                embedded_sdmmc::Mode::ReadWriteCreateOrAppend,
            )
            .map_err(drop)?;
        let buffer = b"test louis\n";
        let _ = volume_mgr.write(&mut volume0, &mut my_file, buffer);
        volume_mgr.close_file(&volume0, my_file).map_err(drop)?;
        Ok(())
    }
}

struct Clock;

impl TimeSource for Clock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
