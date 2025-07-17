#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::vec;
use rust_os::interrupts::CHARACTER;

entry_point!(kernel_main);

mod vga_buffer;
mod serial;

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    STOP,
}

struct ObjPos {
    x: usize,
    y: usize,
    symbol: u8,
    direction: Direction,
}

impl ObjPos {
    fn create_player() -> ObjPos {
        ObjPos {
            x: 5,
            y: 5,
            symbol: b'@',
            direction: Direction::STOP
        }
    }
}

#[unsafe(no_mangle)] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    rust_os::init();

    #[cfg(test)]
    test_main();

    let mut player = ObjPos::create_player();

    loop {
        for _ in 0..502800 {}
        vga_buffer::draw_player(player.y, player.x);
        vga_buffer::draw_board(player.x, player.y);
        let mut character = CHARACTER.lock();

        match character.to_ascii_lowercase() {
            'w' => player.direction = Direction::UP,
            'a' => player.direction = Direction::LEFT,
            's' => player.direction = Direction::DOWN,
            'd' => player.direction = Direction::RIGHT,
            _ => {},
        }

        *character = 'u';

        match player.direction {
            Direction::UP => {
                player.y -= 1;
                if player.y <= 0 {player.y = vga_buffer::HEIGHT-2;}
            }
            Direction::DOWN => {
                player.y += 1;
                if player.y >= vga_buffer::HEIGHT-1 {player.y = 1;}
            }
            Direction::RIGHT => {
                player.x += 1;
                if player.x >= vga_buffer::WIDTH-1 {player.x = 1;}
            }
            Direction::LEFT => {
                player.x -= 1;
                if player.x <= 0 {player.x = vga_buffer::WIDTH-2;}
            }
            ref _other => {}
        }
    }

    rust_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
