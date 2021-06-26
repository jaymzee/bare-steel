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
    println!("some numbers: {} {}", 42, 1.337);

    #[cfg(test)]
    test_main();

    loop {}
}

// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// our panic handler in test mode
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
