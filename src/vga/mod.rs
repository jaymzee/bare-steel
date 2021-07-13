pub mod ansi;

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

/// The height of the text buffer
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer
const BUFFER_WIDTH: usize = 80;

lazy_static! {
    /// A global 'Writer' instance that can be used for printing to the
    /// VGA text buffer
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        row: BUFFER_HEIGHT - 1,
        attr: ScreenAttribute::new(Color::LightCyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut ScreenBuffer) },
    });
}

pub fn set_default_attribute(attr: ScreenAttribute) {
    WRITER.lock().attr = attr;
}

pub fn get_default_attribute() -> ScreenAttribute {
    WRITER.lock().attr
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// VGA text mode attribute value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ScreenAttribute(u8);

impl ScreenAttribute {
    pub fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    code: u8,       // ascii code
    attr: ScreenAttribute,
}

/// A structure representing the VGA text buffer
#[repr(transparent)]
struct ScreenBuffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying
/// `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements
/// the `core::fmt::Write trait.
pub struct Writer {
    row: usize,
    column: usize,
    attr: ScreenAttribute,
    buffer: &'static mut ScreenBuffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row;
                let col = self.column;

                self.buffer.chars[row][col].write(ScreenChar {
                    code: byte,
                    attr: self.attr,
                });
                self.column += 1;
                self.move_cursor();
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    /// Does **not** support strings with non-ASCII characters, since they
    /// can't be printed in the VGA text
    /// mode.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let ch = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(ch);
                }
            }
            self.clear_row(self.row);
        }
        self.column = 0;
        self.move_cursor();
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar { code: b' ', attr: self.attr };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn move_cursor(&self) {
        use x86_64::instructions::port::Port;
        let mut addr = Port::new(0x3D4);
        let mut data = Port::new(0x3D5);
        let offset = BUFFER_WIDTH * self.row + self.column;

        unsafe {
            addr.write(0x0F as u8);   // cursor location lo
            data.write(offset as u8);
            addr.write(0x0E as u8);   // cursor location hi
            data.write((offset >> 8) as u8);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn display(s: &str, pos: (u8, u8), attr: ScreenAttribute) {
    let buffer = unsafe { &mut *(0xb8000 as *mut ScreenBuffer) };
    let mut row = (pos.0 - 1) as usize;
    let mut col = (pos.1 - 1) as usize;

    for byte in s.bytes() {
        let code = match byte {
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' => byte,
            // not part of printable ASCII range
            _ => 0xfe,
        };
        if code == b'\n' {
            row += 1;
            col = 0;
        } else {
            let scrn_char = ScreenChar { code, attr };
            buffer.chars[row][col].write(scrn_char);
            col += 1;
        }
    }
}

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
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let scrn_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(scrn_char.code), c);
        }
    });
}
