extern crate rppal;

use std::cmp;
use std::thread;
use std::time::Duration;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::system::DeviceInfo;
use Bits::*;

fn main() {
    print_device_info();

    let mut panel = LedPanel::new(256);

    let mario: Vec<&str> = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 2, 2,
        2, 0, 1, 1, 1, 1, 3, 3, 2, 1, 1, 1, 1, 0, 1, 2, 2, 1, 3, 0, 3, 3, 1, 1, 1, 2, 2, 1, 1, 1,
        3, 3, 3, 3, 1, 1, 0, 0, 3, 3, 3, 2, 3, 3, 3, 2, 2, 1, 2, 1, 3, 3, 3, 3, 2, 2, 2, 2, 2, 1,
        1, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 1, 1, 2, 2, 2, 2, 2, 3, 3, 0, 3, 1, 1, 2, 1,
        2, 1, 3, 3, 2, 3, 3, 3, 0, 0, 1, 1, 3, 3, 3, 3, 1, 1, 0, 2, 1, 1, 1, 1, 3, 0, 0, 3, 0, 2,
        2, 1, 2, 0, 1, 1, 1, 2, 3, 3, 1, 1, 1, 1, 0, 2, 2, 2, 1, 1, 0, 0, 1, 2, 2, 0, 0, 0, 0, 0,
        0, 0, 2, 0, 0, 0, 0, 1, 2, 2, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
        .iter()
        .map(|val| {
            match val {
                0 => "000000", // Background
                1 => "040101", // Boots
                2 => "020100", // Skin
                3 => "030000", // Clothes
                _ => panic!("Invalid color"),
            }
        }).collect();

    panel.convert_and_write(mario.as_slice());
}

struct LedPanel {
    buffer: String, // Using a String as we need arbitrary number of bits, to later be padded with 0's and converted to [u8].
    spi: Spi,
    num_leds: u32,
}

#[derive(Clone)]
enum Bits {
    _0,
    _1,
}

impl LedPanel {
    fn new(num_leds: u32) -> LedPanel {
        let buffer = String::new();
        let bus = Bus::Spi0; // SPI0 bus needs to be enabled. Runs on physical pin: 21, 19, 23, 24, 26.
        let slave = SlaveSelect::Ss0; // Which device (pin) should listen to the SPI bus. We will be using SS0 pins. i.e. physical pin 21, et al.
        let clock_speed = 3 * 1000 * 1000; // Maximum clock frequency (Hz).
        let mode = Mode::Mode0;

        LedPanel {
            buffer,
            spi: Spi::new(bus, slave, clock_speed, mode).unwrap(),
            num_leds,
        }
    }

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

        self.spi.write(&output).unwrap();
        self.clear_buffer();
    }

    fn convert_and_push(&mut self, hex_codes: &[&str]) {
        let matrix: Vec<Bits> = hex_codes
            .iter()
            .map(|hex_code| LedPanel::hex_to_bin(hex_code))
            .collect::<String>()
            .chars()
            .map(|chr| match chr {
                '0' => _0,
                '1' => _1,
                _ => panic!("Invalid character trying to convert to enum types: {}", chr),
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
        let clear_codes = vec![_0; (self.num_leds * 24) as usize];

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

        let r: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(0).take(2).collect());
        let g: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(2).take(2).collect());
        let b: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(4).take(2).collect());

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
