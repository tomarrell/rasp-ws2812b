/*
   Todo:
       Write a Rust program which interacts with the ws2812b over SPI.

   Info:
       ws2812b can take bits at
       f = 400kHz or 800kHz

       To interact over SPI, each LED sequence (3 bytes) intended for the LEDs need to be converted to
       a 9 byte sequence. This is because each bit gets turned into 3 SPI bits, and the frequency of the
       SPI interface set to 3x that which the LED is expecting.

       This turns out to look a little like this:

       PWM LED (1): where
           on time  = 0.8us +/-150ns
           off time = 0.4us +/-150ns

           |    START                      END
           |    v                          v
           |    ------------------
           |    |                |
           |    |                |
           |    |                |
           | ----                -----------
           *--------------------------------

        PWM LED (0): where
            on time  = 0.4us +/-150ns
            off time = 0.8us +/-150ns

            |    START                      END
            |    v                          v
            |    ----------
            |    |        |
            |    |        |
            |    |        |
            | ----        -------------------
            *--------------------------------

        With SPI breaking this instead into 3 time sections. Where each can be pulled high or low individually.
        We can therefore represent what would have been a PWM (1) with an SPI 110. And on the contrary,
        represent a PWM (0) with an SPI 100.
*/

extern crate rppal;

use std::thread;
use std::time::Duration;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::system::DeviceInfo;

fn main() {
    let device_info = DeviceInfo::new().unwrap();
    println!(
        "Model: {} (SoC: {})",
        device_info.model(),
        device_info.soc()
    );

    // SPI0 bus needs to be enabled. Runs on physical pin: 21, 19, 23, 24, 26.
    let bus = Bus::Spi0;

    // Which device (pin) should listen to the SPI bus. We will be using SS0 pins. i.e. physical pin 21, et al.
    let slave = SlaveSelect::Ss0;

    // Maximum clock frequency (Hz).
    let clock = 3 * 1000 * 1000; // 800 kHz
    let mode = Mode::Mode0;

    let mut panel = Spi::new(bus, slave, clock, mode).unwrap();

    // Send init frame

    // Makes first one green!!!
    let led_buffer = [0b11011011u8, 0b01101101u8, 0b10110110u8];

    // let led_buffer = [1, 1, 1];

    let result = panel.write(&led_buffer);
    println!("{:?}", result);

    thread::sleep(Duration::from_millis(50));
}
