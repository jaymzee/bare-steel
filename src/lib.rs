#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

//#[macro_use]    // for format! macro
extern crate alloc;

pub mod allocator;
pub mod ansi;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod pit;
pub mod serial;
pub mod task;
pub mod vga;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    // unreachable code: since exit_qemu already called by test runner
    hlt_loop(); // just in case test runner is broken
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print_test_name(core::any::type_name::<T>());
        self();
        print_test_passed();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    print_test_failed();
    serial_println!("\x1b[31;1mError\x1b[0m: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

pub fn print_test_name(name: &str) {
    serial_print!("{} ... ", name);
}
pub fn print_test_passed() {
    serial_println!("\x1b[32;1m{}\x1b[0m", "ok");
}

pub fn print_test_failed() {
    serial_println!("\x1b[31;1m{}\x1b[0m\n", "FAILED");
}

pub fn print_test_failed_because(msg: &str) {
    serial_println!("\x1b[31;1m{}\x1b[0m\n", "FAILED");
    serial_println!("\x1b[31;1mError\x1b[0m: {}\n", msg);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    hlt_loop(); // spin here while waiting for qemu to exit
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

