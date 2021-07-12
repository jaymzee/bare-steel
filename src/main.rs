// based on https://os.phil-opp.com/minimal-rust-kernel/
//

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]    // for format! macro
extern crate alloc;

use blog_os::{println, task::timer};
use blog_os::vga::{self, Color, ScreenAttribute};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use blog_os::task::{Task, executor::Executor, keyboard};
    use x86_64::VirtAddr;

    println!("Concurrency with Async/Await");
    vga::set_default_attribute(
        ScreenAttribute::new(Color::LightGray, Color::Black)
    );

    // load GDT, IDT and enable interrupts
    println!("initialize GDT and interrupts...");
    blog_os::init();

    // initialize global allocator
    println!("initialize allocator...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("setting timer tick interval to 20 Hz");
    blog_os::pit::set_interval_timer(20);

    #[cfg(test)]
    test_main();

    println!("initializing tasks...");
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    for id in 0..=6 {
        executor.spawn(Task::new(display_timer(id)));
    }
    executor.spawn(Task::new(display_seconds(7)));

    println!("running tasks...");
    executor.run();
}

async fn display_timer(id: usize) {
    let color = ScreenAttribute::new(Color::LightCyan, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);

    loop {
        let timer = timer::Timer::Tick(id).await;
        vga::display(&format!("{:>6}", timer), scrn_pos, color);
        let timer = timer::Timer::Tock(id).await;
        vga::display(&format!("{:>6}", timer), scrn_pos, color);
    }
}

async fn display_seconds(id: usize) {
    let color = ScreenAttribute::new(Color::Yellow, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);

    for seconds in 0..u32::MAX {
        vga::display(&format!("{:>6}", seconds), scrn_pos, color);
        timer::sleep(id, 20).await;
    }
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
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

