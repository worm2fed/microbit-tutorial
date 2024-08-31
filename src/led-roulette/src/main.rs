#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::{hal::prelude::*, Board};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
    board.display_pins.col3.set_low().unwrap();
    board.display_pins.row3.set_high().unwrap();

    // infinite loop; just so we don't leave this stack frame
    loop {}
}
