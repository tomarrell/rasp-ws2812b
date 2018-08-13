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

use std::cmp;
use std::thread;
use std::time::Duration;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::system::DeviceInfo;
use Bits::*;


fn main() {
    print_device_info();

    let mut panel = LED_Panel::new(256);

    // panel.clear_all_leds();
    // panel.convert_and_show(&["000000", "000000"]);
    // thread::sleep(Duration::from_millis(1000));


    for x in 0..10 {
        panel.clear_all_leds();

        panel.convert_and_write(&["330000", "003300"]);

        thread::sleep(Duration::from_millis(500));
        panel.clear_all_leds();
        thread::sleep(Duration::from_millis(500));

        panel.convert_and_write(&["330000", "003300", "330000", "003300"]);
        thread::sleep(Duration::from_millis(500));
        panel.clear_all_leds();
        thread::sleep(Duration::from_millis(500));

        panel.convert_and_write(&["330000", "003300", "330000", "003300", "330000", "003300"]);
        thread::sleep(Duration::from_millis(500));
        panel.clear_all_leds();
        thread::sleep(Duration::from_millis(500));
    }
}

struct LED_Panel {
    buffer: String, // Using a String as we need arbitrary number of bits, to later be padded with 0's and converted to [u8].
    bus: Bus,
    slave: SlaveSelect,
    clock_speed: u32,
    mode: Mode,
    spi: Spi,
    num_leds: u32,
}

#[derive(Clone)]
enum Bits {
    _0,
    _1,
}

impl LED_Panel {
    fn new(num_leds: u32) -> LED_Panel {
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
            num_leds,
        }
    }

    // Push to bit buffer.
    fn push(&mut self, bits: &str) -> &str {
        self.buffer.push_str(bits);
        &self.buffer
    }

    fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    fn write(&mut self) {
        // Pad with zeroes
        if self.buffer.len() % 8 != 0 {
            let buf_len = self.buffer.len();
            self.buffer.push_str(&"0".repeat(8 - (buf_len % 8)));
        }

        let buffer = self.buffer.clone();
        let mut cur = buffer.as_str();
        let mut v = vec![];
        while !cur.is_empty() {
            let (chunk, rest) = cur.split_at(cmp::min(8, cur.len()));
            v.push(chunk);
            cur = rest;
        }

        let output = v
            .iter()
            .map(|val| u8::from_str_radix(val, 2).unwrap())
            .collect::<Vec<u8>>();

        self.spi.write(&output);
        self.clear_buffer();
    }

    fn convert_and_push(&mut self, hex_codes: &[&str]) {
        let matrix: Vec<Bits> = hex_codes
            .iter()
            .map(|hex_code| LED_Panel::hex_to_bin(hex_code))
            .collect::<String>()
            .chars()
            .map(|chr| {
                match chr {
                    '0' => _0,
                    '1' => _1,
                    _ => panic!("Invalid character trying to convert to enum types: {}", chr),
                }
            })
            .collect();

        for val in matrix.iter() {
            match val {
                Bits::_1 => self.push("110"),
                Bits::_0 => self.push("100"),
            };
        }
    }

    // Push to the buffer and write out
    fn convert_and_write(&mut self, hex_codes: &[&str]) {
        self.convert_and_push(hex_codes);
        self.write();
    }

    // Turns all LEDs off and clears buffer
    fn clear_all_leds(&mut self) {
        self.clear_buffer();
        let clear_codes = vec![_0; (self.num_leds * 8) as usize];

        for val in clear_codes.iter() {
            match val {
                Bits::_1 => self.push("110"),
                Bits::_0 => self.push("100"),
            };
        }

        self.write();
    }

    // Hex string length should be 6
    fn hex_to_bin(hex: &str) -> String {
        if hex.len() != 6 {
            panic!("Hex length must be 6");
        }

        let g: u8 = LED_Panel::hex_str_to_u8(hex.chars().skip(0).take(2).collect());
        let r: u8 = LED_Panel::hex_str_to_u8(hex.chars().skip(2).take(2).collect());
        let b: u8 = LED_Panel::hex_str_to_u8(hex.chars().skip(4).take(2).collect());

        format!("{:08b}{:08b}{:08b}", g, r, b)
    }

    fn hex_str_to_u8(hex_str: String) -> u8 {
        u8::from_str_radix(&hex_str, 16).unwrap()
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
