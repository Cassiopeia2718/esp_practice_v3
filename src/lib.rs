#![no_std]
use esp_hal::time::{Duration, Instant};
use esp_hal::gpio::Output;
extern crate alloc;
use alloc::vec::Vec;
use esp_println::println;

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

pub fn print_csv(readings: &Vec<Reading>) {
    println!("co2, temperature, humidity, time");
    for r in readings {
        println!("{},{},{},{}", r.co2, r.temperature, r.humidity, r.time);
    }
}

pub struct Reading {
    pub co2: u16,
    pub temperature: f32,
    pub humidity: f32,
    pub time: u64
}

pub struct Status {
    pub recording: bool,
    pub frequency: u16,
    pub start_time_ms: u64, 
}
