#![deny(unsafe_code)]
#![no_main]
#![no_std]

use microbit::hal::prelude::*;
use microbit::hal::Timer;

use cortex_m_rt::entry;
use nb::block;
use nb::Error;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use lsm303agr::{AccelOutputDataRate, AccelScale, Lsm303agr};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

#[entry]
fn main() -> ! {
    const THRESHOLD: f32 = 0.5;

    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c =
        { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = {
        twim::Twim::new(
            board.TWIM0,
            board.i2c_internal.into(),
            FREQUENCY_A::K100,
        )
    };

    let mut timer = Timer::new(board.TIMER0);
    let mut delay = Timer::new(board.TIMER1);
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_accel_scale(AccelScale::G16).unwrap();

    let mut max_observed = 0.;
    let mut measuring = false;
    loop {
        while !sensor.accel_status().unwrap().x_new_data {}

        let new_observation = sensor.accel_data().unwrap().x as f32 / 1000.0;
        if measuring {
            match timer.wait() {
                Ok(_) => {
                    rprintln!("Accel max: {}", max_observed);
                    max_observed = 0.;
                    measuring = false;
                }
                Err(Error::WouldBlock) => {
                    max_observed = new_observation.max(max_observed);
                }
                Err(Error::Other(_)) => {
                    unreachable!()
                }
            }
        } else if new_observation > THRESHOLD {
            rprintln!("Measuring!");
            measuring = true;
            max_observed = new_observation;
            // The documentation notes that the timer works at a frequency
            // of 1 Mhz, so in order to wait for 1 second we have to
            // set it to 1_000_000 ticks.
            timer.start(1_000_000_u32);
        }

        delay.delay_ms(20_u8);
    }
}
