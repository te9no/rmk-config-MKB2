#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rmk::macros::rmk_keyboard;

#[rmk_keyboard]
mod keyboard {}
