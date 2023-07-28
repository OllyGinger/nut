#![no_std]
#![no_main]

use core::arch::global_asm;

mod fail;

global_asm!(include_str!("boot.s"));
