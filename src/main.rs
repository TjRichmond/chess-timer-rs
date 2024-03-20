#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::{dma::NoDma, gpio::*, spi::*, time::Hertz};
use cortex_m_rt::entry;
use max7219::*;

#[entry]
fn main() -> ! {
    // create peripheral access crate variable 
    let pac = embassy_stm32::init(Default::default());

    // create spi conf and operation frequency
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(10_000);

    // initialize spi and cs pins
    let my_spi = Spi::new(pac.SPI1, pac.PA5, pac.PA7, pac.PA6, NoDma, NoDma, spi_config);
    let cs_display = Output::new(pac.PD14, Level::High, Speed::VeryHigh);

    // create max7219 instance based on spi
    let mut my_display = MAX7219::from_spi_cs(1, my_spi, cs_display).unwrap();

    // power on display and set to 50% brightness
    my_display.power_on().unwrap();
    my_display.set_intensity(0,0x07).unwrap();

    // infinite loop
    loop {
        // write F to all digits on display
        my_display.write_hex(0, 0xFFFFFFFF).unwrap();
    }
}
