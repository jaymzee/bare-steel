use core::fmt;
use core::num::ParseIntError;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;
use crate::vga::text;

lazy_static! {
    /// A global 'Writer' instance that can be used for printing to the
    /// VGA text buffer
    ///
    /// Used by the `print!` and `println!` macros.
    pub(crate) static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        row: text::BUFFER_HEIGHT - 1,
        attr: Default::default(),
        buffer: unsafe { &mut *(0xb8000 as *mut text::Buffer) },
    });
}

/// indicate Ansi sequence error with a special character
const ANSI_ERROR: text::Char = text::Char::new(13, text::Attribute::error());

/// A writer type that allows writing ASCII bytes and strings to an underlying
/// `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements
/// the `core::fmt::Write trait.
pub(crate) struct Writer {
    row: usize,
    column: usize,
    attr: text::Attribute,
    buffer: &'static mut text::Buffer,
}

impl Writer {
    pub fn clear_screen(&mut self, attr: text::Attribute) {
        self.attr = attr;
        for r in 0..text::BUFFER_HEIGHT {
            self.clear_row(r);
        }
        self.set_cursor_position(1, 1);
    }

    pub fn set_attribute(&mut self, attr: text::Attribute) {
        self.attr = attr;
    }

    pub fn set_cursor_position(&mut self, r: u8, c: u8) {
        self.row = r as usize - 1;
        self.column = c as usize - 1;
        self.move_cursor();
    }

    /// Writes the given ASCII string to the text buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    /// Does **not** support strings with non-ASCII characters, since they
    /// can't be printed in the VGA text
    /// mode. Supports ANSI escape codes for color.
    fn write_string(&mut self, s: &str) {
        let mut state = Ansi::Start;
        let mut arg_start = 0;

        for (i, c) in s.bytes().enumerate() {
            let next_state = match (state, c) {
                (Ansi::Start, b'\x1b') => {
                    Ansi::Esc
                }
                (Ansi::Start, _) => {
                    self.write_byte(c);
                    Ansi::Start
                }
                (Ansi::Esc,  b'[') => {
                    arg_start = i + 1;
                    Ansi::Csi
                }
                (Ansi::Csi, 0x20..=0x3f) => {
                    // CSI parameters and intermediate bytes
                    Ansi::Csi
                }
                (Ansi::Csi, 0x40..=0x7E) => {
                    // final byte of CSI sequence
                    self.write_csi(c, &s[arg_start..i]);
                    Ansi::Start
                }
                _ => {
                    self.write_screen(ANSI_ERROR);
                    Ansi::Start // error happened so better reset state
                }
            };
            state = next_state;
        }
        self.move_cursor();
    }

    /// Writes a ScreenChar to the text buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`.
    fn write_screen(&mut self, ch: text::Char) {
        if self.column >= text::BUFFER_WIDTH {
            self.new_line();
        }

        let row = self.row;
        let col = self.column;

        self.buffer.chars[row][col].write(ch);
        self.column += 1;
    }

    /// Writes an ASCII byte to the text buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x20..=0x7e =>
                self.write_screen(text::Char::new(byte, self.attr)),
            _ =>
                self.write_screen(text::Char::new(0xfe, self.attr)),
        }
    }

    /// Writes an ansi CSI sequence to the text buffer.
    ///
    /// Supports the SGR (select graphic rendition) and
    /// CUP (Cursor Update Position) CSI
    fn write_csi(&mut self, n: u8, args: &str) {
        match n {
            b'm' => self.write_sgr(args),
            b'H' => {
                match split(args, ';').as_slice() {
                    [Ok(r), Ok(c)] => self.set_cursor_position(*r, *c),
                    _ => self.write_screen(ANSI_ERROR)
                }
            }
            _ => self.write_screen(ANSI_ERROR),
        }
    }

    /// Writes an ansi SGR sequence to the text buffer.
    ///
    /// Supports setting the foreground and background color
    fn write_sgr(&mut self, args: &str) {
        if args == "" || args == "0" {
            self.attr = Default::default();
        } else {
            for code in split(args, ';') {
                match code {
                    Ok(n) if (30..=37).contains(&n) => {
                        let fg = text::Color::from_ansi(n - 30);
                        let bg = self.attr.background();
                        self.attr = text::Attribute::new(fg, bg);
                    }
                    Ok(n) if (40..=47).contains(&n) => {
                        let bg = text::Color::from_ansi(n - 40);
                        let fg = self.attr.foreground();
                        self.attr = text::Attribute::new(fg, bg);
                    }
                    Ok(_) => self.write_screen(ANSI_ERROR),
                    Err(_) => self.write_screen(ANSI_ERROR),
                }
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        if self.row < text::BUFFER_HEIGHT - 1 {
            self.row += 1;
        } else {
            for row in 1..text::BUFFER_HEIGHT {
                for col in 0..text::BUFFER_WIDTH {
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
        let blank = text::Char::new(b' ', self.attr);
        for col in 0..text::BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Update cursor position in text buffer.
    fn move_cursor(&self) {
        use x86_64::instructions::port::Port;
        let mut addr = Port::new(0x3D4);
        let mut data = Port::new(0x3D5);
        let offset = text::BUFFER_WIDTH * self.row + self.column;

        unsafe {
            addr.write(0x0F as u8);   // cursor location lo
            data.write(offset as u8);
            addr.write(0x0E as u8);   // cursor location hi
            data.write((offset >> 8) as u8);
        }
    }
}

fn split(args: &str, delimiter: char) -> Vec<Result<u8, ParseIntError>> {
    args.split(delimiter)
        .map(|s| s.parse())
        .collect()
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// ansi escape sequence states
#[derive(Debug, Copy, Clone)]
enum Ansi {
    /// parsing regular characters
    Start,
    /// parsing escape sequence
    Esc,
    /// parsing Control Sequence Introducer
    Csi,
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    use text::BUFFER_HEIGHT;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let scrn_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i]
                .read();
            assert_eq!(char::from(scrn_char.code), c);
        }
    });
}
