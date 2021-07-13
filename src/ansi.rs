pub fn parse_ansi(s: &str) {
    let mut state = Ansi::Char;
    let mut index = 0;

    for (i, c) in s.chars().enumerate() {
        let next_state = match state {
            Ansi::Char if c == '\x1b' => {
                Ansi::Esc
            }
            Ansi::Char => {
                print_char(c);
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
            _ => panic!("failed to parse ansi sequence")
        };
        //println!("{:x}: {:?} -> {:?}", c as u32, self.state, next_state);
        state = next_state;
    }
}

fn csi(c: char, args: &str) {
    if c == 'm' {
        sgr(args);
    } else if c == 'H' {
        let args: Vec<u8> = args.split(';')
            .map(|s| s.parse().unwrap())
            .collect();
        println!("CUP {} {}", args[0], args[1]);
    } else {
        panic!( "unknown CSI command {} in ansi sequence", c);
    }
}

fn sgr(args: &str) {
    if args == "" || args == "0" {
        println!("SGR RESET");
    } else {
        let args: Vec<_> = args.split(';')
            .map(|s| s.parse())
            .collect();
        for arg in args {
            let cmd = arg.expect("SGR parse error");

            if cmd == 1 {
                println!("SGR INTENSITY");
            } else if (30..=37).contains(&cmd) {
                let color = cmd - 30;
                println!("SGR FG COLOR {}", color);
            } else if (40..=47).contains(&cmd) {
                let color = cmd - 40;
                println!("SGR BG COLOR {}", color);
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
