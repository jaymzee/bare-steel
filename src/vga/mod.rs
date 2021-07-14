pub mod text;

use core::fmt;
use text::writer;

/// Like the `print!` macro in the standard library, but prints to the
/// VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

/// Like the `print!` macro in the standard library, but prints to the
/// VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text bufer through the
/// global `WRITER` intstance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        writer::WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    crate::println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        crate::println!("test_println_many output");
    }
}
