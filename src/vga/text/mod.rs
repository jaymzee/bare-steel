pub mod writer;

use volatile::Volatile;
use writer::WRITER;

/// The height of the text buffer
pub const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer
pub const BUFFER_WIDTH: usize = 80;

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

impl From<u8> for Color {
    fn from(n: u8) -> Self {
        match n {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
           10 => Color::LightGreen,
           11 => Color::LightCyan,
           12 => Color::LightRed,
           13 => Color::Pink,
           14 => Color::Yellow,
           15 => Color::White,
           _ => Color::Black,
        }
    }
}

impl Color {
    pub fn from_ansi(n: u8) -> Self {
        match n {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            _ => Color::Black,
        }
    }
}

/// VGA text mode attribute value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Attribute(u8);

impl Attribute {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    pub const fn default() -> Self {
        Self::new(Color::LightGray, Color::Black)
    }

    pub const fn error() -> Self {
        Self::new(Color::White, Color::Black)
    }

    pub fn background(self) -> Color {
        match self {
            Self(n) => (n as u8 >> 4).into()
        }
    }

    pub fn foreground(self) -> Color {
        match self {
            Self(n) => (n as u8).into()
        }
    }
}

impl Default for Attribute {
    fn default() -> Self {
        Self::default()
    }
}

/// A screen character in the VGA text buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Char {
    pub code: u8,  // ascii code
    pub attr: Attribute,
}

impl Char {
    pub const fn new(code: u8, attr: Attribute) -> Self {
        Self { code, attr }
    }
}

/// A structure representing the VGA text buffer
#[repr(transparent)]
pub(crate) struct Buffer {
    pub(crate) chars: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Write a string at the row and column in the text buffer
pub fn display(s: &str, pos: (u8, u8), attr: Attribute) {
    let buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };
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
            let scrn_char = Char::new(code, attr);
            buffer.chars[row][col].write(scrn_char);
            col += 1;
        }
    }
}

/// Clear the screen filling the buffer with the attribute (Synchronized)
///
/// Do not call if you already have a mutex lock on WRITER
/// use the equivalent method on the WRITER instead
pub fn clear_screen(attr: Attribute) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen(attr);
    });
}

/// Set the text buffer attribute for the writer to use (Synchronized)
///
/// Do not call if you already have a mutex lock on WRITER
/// use the equivalent ethod on the WRITER instead
pub fn set_attribute(attr: Attribute) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().set_attribute(attr);
    });
}
