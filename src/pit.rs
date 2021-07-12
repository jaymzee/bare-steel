/* Programmable interval timer 8253/8254
 * oscillator frequency is approximately 1.193182 Mhz
 *
 * Channel 0 connected to PIC IRQ 0 (IRQ generated by low to high transition).
 * Channel 1 not used (legacy DMA refresh)
 * Channel 2 connected to PC speaker.
 */

const PIT_CH0_DATA: u16 = 0x40;   // Channel 0 data port (read/write)
//const PIT_CH1_DATA: u16 = 0x41;   // Channel 1 data port (read/write)
//const PIT_CH2_DATA: u16 = 0x42;   // Channel 2 data port (read/write)
const PIT_MODE_CMD: u16 = 0x43;   // mode/command register (write only)

const PIT_CLK_SRC: u32 = 1193182;

pub fn set_interval_timer(frequency: u32) {
    use x86_64::instructions::port::Port;

    // Set the PIT ch0 to the desired frequency
    let divider = PIT_CLK_SRC / frequency;
    let mut data = Port::new(PIT_CH0_DATA);
    let mut cmd = Port::new(PIT_MODE_CMD);

    unsafe {
        cmd.write(0x34 as u8); // lobyte/highbyte, rate generator
        data.write(divider as u8);
        data.write((divider >> 8) as u8);
    }
}