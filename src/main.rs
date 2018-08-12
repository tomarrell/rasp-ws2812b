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
use std::cmp;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::system::DeviceInfo;

enum Bits {
    _0,
    _1,
}

fn main() {
    print_device_info();

    let mut panel = LED_Panel::new();

    use Bits::*;
    let matrix: Vec<Bits> = vec![
        _0, _0, _0, _0, _0, _0, _0, _1,
        _0, _0, _0, _0, _0, _0, _0, _1,
        _0, _0, _0, _0, _0, _0, _0, _1,
    ];

    for val in matrix.iter() {
        match val {
            Bits::_1 => panel.push("110"),
            Bits::_0 => panel.push("100"),
            _ => panic!("Matrix value not recognized"),
        };
    }

    println!("{}", panel.buffer);
    panel.write();
}

struct LED_Panel {
    buffer: String, // Using a String as we need arbitrary number of bits, to later be padded with 0's and converted to [u8].
    bus: Bus,
    slave: SlaveSelect,
    clock_speed: u32,
    mode: Mode,
    spi: Spi,
}

impl LED_Panel {
    fn new() -> LED_Panel {
        let buffer = String::new();
        let bus = Bus::Spi0; // SPI0 bus needs to be enabled. Runs on physical pin: 21, 19, 23, 24, 26.
        let slave = SlaveSelect::Ss0; // Which device (pin) should listen to the SPI bus. We will be using SS0 pins. i.e. physical pin 21, et al.
        let clock_speed = 3 * 1000 * 1000; // Maximum clock frequency (Hz).
        let mode = Mode::Mode0;

        LED_Panel {
            buffer,
            bus,
            slave,
            clock_speed,
            mode,
            spi: Spi::new(bus, slave, clock_speed, mode).unwrap(),
        }
    }

    // Push to bit buffer.
    fn push(&mut self, bits: &str) -> &str {
        self.buffer.push_str(bits);
        &self.buffer
    }

    fn write(&mut self) {
        // Pad with zeroes
        if self.buffer.len() % 8 != 0 {
            let buf_len = self.buffer.len();
            self.buffer.push_str(&"0".repeat(8 - (buf_len % 8)));
        }

        let mut v = vec![];
        let mut cur = self.buffer.as_str();
        let sub_len = 8;
        while !cur.is_empty() {
            let (chunk, rest) = cur.split_at(cmp::min(sub_len, cur.len()));
            v.push(chunk);
            cur = rest;
        }

        let output = v.iter().map(|val| {
            u8::from_str_radix(val, 2).unwrap()
        }).collect::<Vec<u8>>();

        self.spi.write(&output);
    }
}

fn print_device_info() {
    let device_info = DeviceInfo::new().unwrap();
    println!(
        "Model: {} (SoC: {})",
        device_info.model(),
        device_info.soc()
    );
}
