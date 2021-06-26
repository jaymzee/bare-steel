// based on https://os.phil-opp.com/minimal-rust-kernel/
//
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

fn locate_text(row: u8, col: u8, s: &[u8]) {
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &byte) in s.iter().enumerate() {
        unsafe {
            let dx = row as isize * 80 + col as isize + i as isize;
            *vga_buffer.offset(2 * dx) = byte;
            *vga_buffer.offset(2 * dx + 1) = 0xb; // set attribute
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    locate_text(0, 0, b"Hello, World!");
    locate_text(1, 5, b"Goodbye");

    loop {}
}

// this function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
