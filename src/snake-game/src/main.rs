#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::Board;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();

    loop {}
}
