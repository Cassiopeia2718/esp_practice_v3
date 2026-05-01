#![no_std]
#![allow(unused_imports)]
extern crate alloc;
use alloc::{vec::Vec, format};
use defmt::{info, Debug2Format};
use esp_hal::gpio::{Level, Output, OutputConfig}; 
use esp_hal::i2c::master::{Config as i2c_Config, I2c}; 
use esp_hal::uart::{Uart, Config as uart_Config};
use esp_hal::{clock::CpuClock, delay::Delay, main, time::{Duration, Rate, Instant}};
use scd4x::Scd4x;
use {esp_backtrace as _, esp_println::println};
use esp_hal::Blocking;

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
        Ok(_) => {
            let msg = format!("Read Command");
            uart.write(msg.as_bytes()).ok();
        }
        Err(e) => {
            let msg = format!("Uart Error: {}", e);
            uart.write(msg.as_bytes()).ok();
        }
    }
    buf[0]
}

pub fn send_data(uart: &mut Uart<'_, Blocking>, p: &mut usize, readings: &Vec<Reading>) -> () {
    let k: usize = *p - 1;
    if k == readings.len() {return ();}
    for i in k..=readings.len() { 
        let msg = format!("{},{},{},{}", readings[i].co2, readings[i].temperature, readings[i].humidity, readings[i].time);
        uart.write(msg.as_bytes()).ok();
    }
    *p = readings.len();
}

pub fn collect_data(readings: &mut Vec<Reading>, e_status: &Status, uart: &mut Uart<'_, Blocking>, s: &mut bool, scd41: &mut Scd4x<I2c<'_, esp_hal::Blocking>, Delay>) -> () {
    match &scd41.data_ready_status() {
        Ok(status) => {
            *s = *status;
        },
        Err(e) => {
            let msg = format!("Data ready error: {}", e);
            uart.write(msg.as_bytes()).ok();
        },
    }

    if !*s {return ();}
    *s = false;

    match &scd41.measurement() {    
        Ok(data) => {
            readings.push(Reading {co2: data.co2, temperature: data.temperature, humidity: data.humidity, time: Instant::now().duration_since_epoch().as_millis() - e_status.start_time_ms});            
        },
        Err(e) => {
            let msg = format!("Measurment Error: {}", e);
            uart.write(msg.as_bytes()).ok();
        },
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
