use crate::{vga, print, println};
use alloc::vec::Vec;
use core::num::ParseIntError;

const PARSE_ERR: &'static str = "failed to parse ansi sequence";

pub fn write_str(s: &str) {
    let mut state = Ansi::Char;
    let mut index = 0;

    for (i, c) in s.chars().enumerate() {
        let next_state = match state {
            Ansi::Char if c == '\x1b' => {
                Ansi::Esc
            }
            Ansi::Char => {
                print!("{}", c);
                Ansi::Char
            }
            Ansi::Esc if c == '[' => {
                index = i + 1;
                Ansi::Csi
            }
            Ansi::Csi if (0x20..=0x3f).contains(&(c as u32)) => {
                // parameters and intermediate bytes
                Ansi::Csi
            }
            Ansi::Csi if (0x40..=0x7E).contains(&(c as u32)) => {
                // final byte
                csi(c, &s[index..i]);
                Ansi::Char
            }
            _ => panic!("{}: state={:?}, char={}", PARSE_ERR, state, c)
        };
        //println!("{:x}: {:?} -> {:?}", c as u32, state, next_state);
        state = next_state;
    }
}

fn parse_args(args: &str, delimiter: char) -> Vec<Result<u8, ParseIntError>> {
    args.split(delimiter)
        .map(|arg| arg.parse())
        .collect()
}

fn csi(c: char, args: &str) {
    if c == 'm' {
        sgr(args);
    } else if c == 'H' {
        let args = parse_args(args, ';');

        if args.len() != 2 {
            panic!("{}: expected ESC[n;mH got ESC[{:?}H", PARSE_ERR, args);
        }

        if let (Ok(row), Ok(column)) = (&args[0], &args[1]) {
            println!("CUP {} {}", row, column);
        } else {
            panic!("{}: expected ESC[n;mH got ESC[{:?}H", PARSE_ERR, args);
        }
    } else {
        panic!("{}: unsupported CSI sequence ESC[{}{}", PARSE_ERR, args, c);
    }
}

fn sgr(args: &str) {
    if args == "" || args == "0" {
        let fg = vga::Color::LightGray;
        let bg = vga::Color::Black;
        let attr = vga::ScreenAttribute::new(fg, bg);
        vga::set_default_attribute(attr);
    } else {
        let cmds = parse_args(args, ';');
        for c in cmds {
            match c {
                Ok(1) => println!("[SGR INTENSITY]"),
                Ok(n) if (30..=37).contains(&n) => {
                    let fg = vga::Color::from_ansi(n - 30);
                    let mut attr = vga::get_default_attribute();
                    attr = vga::ScreenAttribute::new(fg, attr.bg());
                    vga::set_default_attribute(attr);
                }
                Ok(n) if (40..=47).contains(&n) => {
                    let bg = vga::Color::from_ansi(n - 40);
                    let mut attr = vga::get_default_attribute();
                    attr = vga::ScreenAttribute::new(attr.fg(), bg);
                    vga::set_default_attribute(attr);
                }
                Ok(n) => {
                    panic!("{}: bad arg {} in ESC[{}m", PARSE_ERR, n, args);
                }
                Err(e) => {
                    panic!("{}: bad args in ESC[{}m: {}", PARSE_ERR, args, e);
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Ansi {
    Char,   // regular characters
    Esc,    // in an escape sequence
    Csi,    // in a Control Sequence Introducer
}
