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
use blog_os::vga::text;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use blog_os::task::{executor::Executor, Task, keyboard};
    use x86_64::VirtAddr;

    text::clear_screen(Default::default());

    // load GDT, IDT and enable interrupts
    println!("\n\nloading GDT and enabling interrupts...");
    blog_os::init();

    // initialize global allocator
    println!("initializing heap allocator...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("setting timer tick to 18.2 Hz");
    timer::pit::set_divider(timer::pit::Chan::CH0, u16::MAX);

    #[cfg(test)]
    test_main();

    // do not use ansi until heap allocator is initialized
    println!("\x1b[20;1Hansi color \x1b[32mgreen\x1b[0m \
             and \x1b[31mred\x1b[0m text!");

    println!("spawning tasks...");
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    for id in 0..5 {
        executor.spawn(Task::new(display_timer(id)));
    }
    executor.spawn(Task::new(serial_sender(5)));
    executor.spawn(Task::new(display_random(6)));
    executor.spawn(Task::new(display_seconds(7)));

    executor.run();
}

async fn display_timer(id: usize) {
    use text::Color;
    let color = text::Attribute::new(Color::LightCyan, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);

    loop {
        let timer = timer::Timer::Tick(id).await;
        text::display(&format!("{:>6}", timer), scrn_pos, color);
        let timer = timer::Timer::Tock(id).await;
        text::display(&format!("{:>6}", timer), scrn_pos, color);
    }
}

async fn display_seconds(id: usize) {
    use text::Color;
    let color = text::Attribute::new(Color::Yellow, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);

    for seconds in 0..u32::MAX {
        text::display(&format!("{:>6}", seconds), scrn_pos, color);
        timer::sleep(id, 18).await;
    }
}

async fn display_random(id: usize) {
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use text::Color;

    let color = text::Attribute::new(Color::Green, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);
    let mut rng = SmallRng::seed_from_u64(42);

    loop {
        let num: u8 = rng.gen();
        text::display(&format!("{:>6}", num), scrn_pos, color);
        timer::sleep(id, 9).await;
    }
}

async fn serial_sender(id: usize) {
    use blog_os::serial_println;
    use text::Color;
    let color = text::Attribute::new(Color::Blue, Color::Black);
    let scrn_pos = (1, 3 + 8 * id as u8);

    for seconds in 0..u32::MAX {
        serial_println!("greetings {}", seconds);
        text::display(&format!("{:>6}", seconds), scrn_pos, color);
        timer::sleep(id, 18).await;
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

