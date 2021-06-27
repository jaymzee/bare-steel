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
    println!("Hardware Interrupts");
    set_text_attr(TextAttribute::new(Color::LightGray, Color::Black));

    blog_os::init();

    #[cfg(test)]
    test_main();

    for i in 1..20 {
        println!("i = {}", i);
    }

    blog_os::hlt_loop();
}

// panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    blog_os::hlt_loop();
}

// panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info);

    blog_os::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn basic_assertion() {
    assert_eq!(2, 2);
}
