#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::SPI1;
use embassy_stm32::{dma::NoDma, exti::ExtiInput, gpio::*, spi::*, time::Hertz};
use embassy_time::{Duration, Ticker, Timer};
use max7219::*;
use {defmt_rtt as _, panic_probe as _};

static TURN_STATE: AtomicBool = AtomicBool::new(true);

#[embassy_executor::task]
async fn read_turn_button(mut buttons: [ExtiInput<'static>; 2]) {
    loop {
        for button in buttons.iter_mut() {
            button.wait_for_rising_edge().await;
            println!("Button Level: {}", button.get_level());
            TURN_STATE.fetch_xor(true, Ordering::SeqCst);
            TURN_STATE.load(Ordering::SeqCst);
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // create peripheral access crate variable
    let p = embassy_stm32::init(Default::default());

    let black_button = ExtiInput::new(p.PF14, p.EXTI14, Pull::Down);
    let white_button = ExtiInput::new(p.PF15, p.EXTI15, Pull::Down);
    
    let mut test_signal = Output::new(p.PF13,Level::Low,Speed::VeryHigh);

    let buttons: [ExtiInput<'static>; 2] = [white_button, black_button];

    // create spi conf and operation frequency
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_300);
    let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, NoDma, NoDma, spi_config);
    let cs_display = Output::new(p.PD14, Level::High, Speed::VeryHigh);
    let mut display: MAX7219<connectors::SpiConnectorSW<Spi<'_, SPI1, NoDma, NoDma>, Output<'_>>> =
        MAX7219::from_spi_cs(1, spi, cs_display).unwrap();

    display.power_on().unwrap();
    display.set_intensity(0, 0x01).unwrap();

    let mut white_time: i32 = 500;
    let mut black_time: i32 = 500;
    let mut white_ticks: i32 = 0;
    let mut black_ticks: i32 = 0;

    let mut total_time: i32 = 0;

    spawner.spawn(read_turn_button(buttons)).unwrap();
    loop {
        if black_time != 0 && white_time != 0 {
            if TURN_STATE.load(Ordering::Relaxed) == false {
                Timer::after_millis(1).await;
                black_ticks += 1;
                if black_ticks % 50 == 0 {
                    if black_time > 0 {
                        if black_time % 100 == 0 {
                            black_time -= 100;
                            black_time += 59;
                        } else {
                            black_time -= 1;
                        }
                    } else {
                        black_time = 0;
                    }
                }
            } else {
                Timer::after_millis(1).await;
                white_ticks += 1;
                if white_ticks % 50 == 0 {
                    if white_time > 0 {
                        if white_time % 100 == 0 {
                            white_time -= 100;
                            white_time += 59;
                        } else {
                            white_time -= 1;
                        }
                    } else {
                        white_time = 0;
                    }
                }
            }
            total_time = white_time * 10000 + black_time;
            display.write_integer(0, total_time).unwrap();
        }
        test_signal.toggle();
    }
}
