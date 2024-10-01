#![no_main]
#![no_std]

mod controls;
mod display;
mod game;

use cortex_m_rt::entry;
use display::clear_display;
use microbit::{
    display::nonblocking::{BitImage, GreyscaleImage},
    hal::{prelude::*, Rng, Timer},
    Board,
};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use crate::controls::{get_turn, init_buttons};
use crate::display::{display_image, init_display};
use crate::game::{Game, GameStatus};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0).into_periodic();
    let mut rng = Rng::new(board.RNG);
    let mut game = Game::new(rng.random_u32());

    init_buttons(board.GPIOTE, board.buttons);
    init_display(board.TIMER1, board.display_pins);

    loop {
        loop {
            // Game loop
            let image = GreyscaleImage::new(&game.game_matrix(6, 3, 9));
            display_image(&image);
            timer.delay_ms(game.step_len_ms());
            match game.status {
                GameStatus::Ongoing => game.step(get_turn(true)),
                _ => {
                    for _ in 0..3 {
                        clear_display();
                        timer.delay_ms(200u32);
                        display_image(&image);
                        timer.delay_ms(200u32);
                    }
                    clear_display();
                    display_image(&BitImage::new(&game.score_matrix()));
                    timer.delay_ms(2000u32);
                    break;
                }
            }
        }
        game.reset();
    }
}
