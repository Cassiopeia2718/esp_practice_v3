#![no_std]
#![no_main]
use esp_hal::time::{Duration, Instant};
use esp_hal::gpio::Output;

pub fn _delay_ms(milli: u64) -> () {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(milli) {}
}

pub fn _pulse_led(led: &mut Output<'_>, n: u32) -> () {
    for _ in 0..n {
        led.toggle();
        _delay_ms(250);
        led.toggle();
        _delay_ms(250);

    }
}