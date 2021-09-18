/* Programmable interval timer 8253/8254
 * oscillator frequency is approximately 1.193182 Mhz
 *
 * data registers:
 * Channel 0 40h connected to PIC IRQ 0 (IRQ caused by low to high transition)
 * Channel 1 41h not used (legacy DMA refresh)
 * Channel 2 42h connected to PC speaker.
 * mode/command register 43h
 */

use core::convert::TryInto;
use x86_64::instructions::port::Port;

const CLK_FREQ: u32 = 1193182;

pub fn set_frequency(ch: Chan, freq: u32) {
    let clk_div = (CLK_FREQ / freq).try_into()
        .expect("failed to set timer frequency (too low)");

    set_divider(ch, clk_div);
}

pub fn set_divider(ch: Chan, div: u16) {
    let mut data = Port::new(0x40 + ch as u16);
    let mut cmd = Port::new(0x43);  // mode/command (write only)

    unsafe {
        cmd.write(0x34 as u8);      // lobyte/highbyte, rate generator
        data.write(div as u8);
        data.write((div >> 8) as u8);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Chan {
    CH0,
    CH1,
    CH2,
}
