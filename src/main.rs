// based on https://os.phil-opp.com/minimal-rust-kernel/
//

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    use blog_os::vga_buffer::{Color, TextAttribute, set_text_attr};
    set_text_attr(TextAttribute::new(Color::LightCyan, Color::Black));
    println!("Double Faults");

    blog_os::init();

    #[cfg(not(test))]
    cause_page_fault();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    loop {}
}

fn cause_page_fault() {
    unsafe {
        let p = 0xdeadbeef as *mut i32;
        *p = 42;
    };
}

// panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info);
    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn basic_assertion() {
    assert_eq!(2, 2);
}
