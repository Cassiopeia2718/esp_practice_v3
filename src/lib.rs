#![no_std]
use esp_hal::Blocking;
use esp_hal::time::{Duration, Instant};
use esp_hal::{gpio::Output, uart::Uart};
extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use defmt::info;

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

pub fn read_command(uart: &mut Uart<'_, Blocking>) -> u8 {
    let mut buf = [0u8; 1];       

    if !uart.read_ready(){return 0 as u8;}

    match uart.read(&mut buf) {
        Ok(_) => info!("Read command"),
        Err(e) => info!("Uart Error: {}", e)
    }
    buf[0]
}

pub fn send_data(uart: &mut Uart<'_, Blocking>, p: &mut usize, readings: &Vec<Reading>) -> () {
    let k: usize = *p;
    if k == readings.len() {return ();}
    for i in k..=readings.len() { 
        let msg = format!("{},{},{},{}", readings[i].co2, readings[i].temperature, readings[i].humidity, readings[i].time);
        uart.write(msg.as_bytes()).ok();
    }
    *p = readings.len();
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
