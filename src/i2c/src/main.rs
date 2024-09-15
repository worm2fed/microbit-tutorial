#![no_main]
#![no_std]

use core::{fmt::Write, str};
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};
use microbit::hal::prelude::*;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    hal::uart,
    hal::uart::{Baudrate, Parity},
    pac::twi0::frequency::FREQUENCY_A,
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
    pac::twim0::frequency::FREQUENCY_A,
};
#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

// use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let mut serial = {
        uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };
    #[cfg(feature = "v1")]
    let mut i2c =
        { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };
    #[cfg(feature = "v2")]
    let i2c = {
        twim::Twim::new(
            board.TWIM0,
            board.i2c_internal.into(),
            FREQUENCY_A::K100,
        )
    };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut buffer: Vec<u8, 16> = Vec::new();
    loop {
        buffer.clear();
        loop {
            let byte = nb::block!(serial.read()).unwrap();
            nb::block!(serial.write(byte)).unwrap();

            if byte == 13 {
                match str::from_utf8(&buffer) {
                    Ok("accelerometer") => {
                        if sensor.accel_status().unwrap().xyz_new_data {
                            let data = sensor.accel_data().unwrap();
                            write!(
                                serial,
                                "x = {}, y = {}, z = {}\r\n",
                                data.x, data.y, data.z
                            )
                            .unwrap();
                        }
                    }
                    Ok("magnetometer") => {
                        if sensor.mag_status().unwrap().xyz_new_data {
                            let data = sensor.mag_data().unwrap();
                            write!(
                                serial,
                                "x = {}, y = {}, z = {}\r\n",
                                data.x, data.y, data.z
                            )
                            .unwrap();
                        }
                    }
                    Ok(command) => {
                        write!(serial, "Invalid command: {}\r\n", command)
                            .unwrap()
                    }
                    Err(e) => write!(serial, "Error: {}\r\n", e).unwrap(),
                }
                break;
            } else if buffer.push(byte).is_err() {
                write!(serial, " <- Buffer full, flushing\r\n");
                break;
            }
            nb::block!(serial.flush());
        }
    }
}
