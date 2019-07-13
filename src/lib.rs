use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

mod wire_protocol;
use wire_protocol::byte_to_spi_bytes;

pub struct LedPanel {
    /// stores [g, r, b] for each led (as opposed to the normal RGB)
    buffer: Vec<u8>,
    spi: Spi,
    num_leds: u32,
}

/// Stores color as a tuple of (Red, Green, Blue)
pub struct ColorRGB (pub u8, pub u8, pub u8);

impl LedPanel {
    pub fn new(num_leds: u32) -> LedPanel {
        // SPI0 bus needs to be enabled.
        // Runs on physical pin: 21, 19, 23, 24, 26.
        let bus = Bus::Spi0;
        let mode = Mode::Mode0;

        // Which device (pin) should listen to the SPI bus.
        // We will be using SS0 pins. i.e. physical pin 21, et al.
        let slave = SlaveSelect::Ss0;

        let clock_speed = 3 * 1000 * 1000;
        let buffer = Vec::new();

        LedPanel {
            buffer,
            spi: Spi::new(bus, slave, clock_speed, mode).unwrap(),
            num_leds,
        }
    }

    fn write(&mut self) {
        let output = self.buffer
            .drain(..)
            .flat_map(|val| byte_to_spi_bytes(val).to_vec())
            .collect::<Vec<u8>>();

        self.spi.write(&output).unwrap();
    }

    pub fn set_leds(&mut self, hex_codes: &[ColorRGB]) {
        hex_codes
            .iter()
            .for_each(|hex_code| {
                // swapping here from RGB to the GRB expected by the LED panel
                self.buffer.extend_from_slice(&[hex_code.1, hex_code.0, hex_code.2]);
            });
        self.write();
    }

    // Turns all LEDs off and clears buffer
    pub fn clear_all_leds(&mut self) {
        self.buffer.clear();
        let mut clear_codes = vec![0; (self.num_leds * 3) as usize];

        self.buffer.append(&mut clear_codes);

        self.write();
    }
}
