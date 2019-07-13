use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct LedPanel {
    /// stores [g, r, b] for each led (as opposed to the normal RGB)
    buffer: Vec<u8>,
    spi: Spi,
    num_leds: u32,
}

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
            .flat_map(|val| LedPanel::byte_to_spi_bytes(val).to_vec())
            .collect::<Vec<u8>>();

        self.spi.write(&output).unwrap();
    }

    // Convert panel bits into their SPI counterparts
    // 0 -> 001
    // 1 -> 011
    fn byte_to_spi_bytes(input: u8) -> [u8; 3] {
        // first convert the u8 to 24 bits
        let mut bool_array = [false; 24];
        for bit_index in 0..8 {
            let bit = input & (1 << bit_index) != 0;
            let out_index = bit_index * 3;

            // first bit is always 0
            // this could be omitted because the array is initialized to false
            bool_array[out_index] = false;

            bool_array[out_index + 1] = bit;

            // last bit is always 1
            bool_array[out_index + 2] = true;
        }

        // then convert the 24 bits to three u8
        [
            LedPanel::bool_slice_to_u8(&bool_array[0..8]),
            LedPanel::bool_slice_to_u8(&bool_array[8..16]),
            LedPanel::bool_slice_to_u8(&bool_array[16..24]),
        ]
    }

    fn bool_slice_to_u8(input: &[bool]) -> u8 {
        if input.len() != 8 { panic!("bool to u8 conversion requires exactly 8 booleans") }

        let mut out = 0b0000_0000u8;

        for (carry_bit, flag) in input.iter().enumerate() {
            if *flag { out += 0b0000_0001u8 << carry_bit }
        }

        out
    }

    // Convert hex code strings to bytes
    // and push them onto the buffer.
    fn convert_and_push(&mut self, hex_codes: &[&str]) {
        hex_codes
            .iter()
            .for_each(|hex_code| {
                let bytes = LedPanel::hex_to_bin(hex_code);
                self.buffer.extend_from_slice(&bytes);
            });
    }

    // Push to the buffer and write out
    pub fn convert_and_write(&mut self, hex_codes: &[&str]) {
        self.convert_and_push(hex_codes);
        self.write();
    }

    // Turns all LEDs off and clears buffer
    pub fn clear_all_leds(&mut self) {
        self.buffer.clear();
        let mut clear_codes = vec![0; (self.num_leds * 3) as usize];

        self.buffer.append(&mut clear_codes);

        self.write();
    }

    // Hex string length should be 6
    fn hex_to_bin(hex: &str) -> [u8; 3] {
        if hex.len() != 6 {
            panic!("Hex length must be 6");
        }

        let r: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(0).take(2).collect());
        let g: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(2).take(2).collect());
        let b: u8 = LedPanel::hex_str_to_u8(hex.chars().skip(4).take(2).collect());

        [r, g, b]
    }

    fn hex_str_to_u8(hex_str: String) -> u8 {
        u8::from_str_radix(&hex_str, 16).unwrap()
    }
}
