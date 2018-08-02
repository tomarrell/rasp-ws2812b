extern crate rppal;

use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, Level, Mode};
use rppal::system::DeviceInfo;

// BCM Pin numbering
const GPIO_LED: u8 = 18;

fn main() {
    let device_info = DeviceInfo::new().unwrap();
    println!(
        "Model: {} (SoC: {})",
        device_info.model(),
        device_info.soc()
    );

    let mut gpio = Gpio::new().unwrap();
    gpio.set_mode(GPIO_LED, Mode::Output);

    // Blink an LED attached to the pin on and off
    gpio.write(GPIO_LED, Level::High);
    thread::sleep(Duration::from_millis(500));
    gpio.write(GPIO_LED, Level::Low);
}
