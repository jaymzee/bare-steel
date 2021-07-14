#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use blog_os::println;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    blog_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {}
}

#[test_case]
fn test_println_ansi_CUP() {
    println!("\x1b[1;1H CUP");
}

#[test_case]
fn test_println_ansi_SGR() {
    println!("some \x1b[31mred\x1b[0m text");
}

#[test_case]
fn test_println_ansi_bad_escape_seq() {
    println!("invalid \x1b] escape");
}

#[test_case]
fn test_println_ansi_bad_CSI() {
    println!("invalid \x1b[z CSI");
}

#[test_case]
fn test_println_ansi_bad_SGR() {
    println!("invalid \x1b[2m SGR");
}

#[test_case]
fn test_println_ansi_bad_CUP_args() {
    println!("bad \x1b[=;1H args");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
