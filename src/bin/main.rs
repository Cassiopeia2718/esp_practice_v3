#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![allow(unused_imports)]

extern crate alloc;
use alloc::vec::Vec;
use defmt::Debug2Format;
use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config as i2c_Config, I2c};
use esp_hal::{delay::Delay, main, time::{Rate, Instant}};
use esp_hal::uart::{Uart, Config as uart_Config};

use scd4x::Scd4x;
use {esp_backtrace as _, esp_println::println};
use esp_practice_v3::{_delay_ms, Reading, Status, print_csv};

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[main]
fn main() -> ! {

    let mut readings: Vec<Reading> = Vec::new();
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    _delay_ms(1000);

    let led_gpio = peripherals.GPIO2;
    let sda_gpio = peripherals.GPIO22;
    let scl_gpio = peripherals.GPIO21;

    // Set GPIO2 as an output, and set its state to high initially.
    let mut _led = Output::new(led_gpio, Level::Low, OutputConfig::default());

    //create i2c
    let slow_clock = i2c_Config::default().with_frequency(Rate::from_khz(10));
    let i2c = I2c::new(peripherals.I2C0, slow_clock)
        .unwrap_or_else(|_| panic!("Failed to initialize I2C"))
        .with_sda(sda_gpio)
        .with_scl(scl_gpio);

    //create scd41
    let mut scd41 = Scd4x::new(i2c, Delay::new());
    
    
    match scd41.self_test_is_ok() {
        Ok(_) => info!("Self Test Ok"),
        Err(e) => info!("Error: {}", Debug2Format(&e)),
    }

    match scd41.start_periodic_measurement() {
        Ok(_) => info!("measurments started"),
        Err(e) => info!("Measurment faild: {}", Debug2Format(&e)),
    }

    let mut s = false;

    let e_status = Status {
        recording: false,
        frequency: 1,
        start_time_ms: Instant::now().duration_since_epoch().as_millis(),
    };

    let mut uart = Uart::new(peripherals.UART0, uart_Config::default())
        .unwrap()
        .with_rx(peripherals.GPIO3)
        .with_tx(peripherals.GPIO1);

    let buf = [0u8; 64];
    let pos = 0;
    let mut single = [0u8; 1];


    loop {
        _delay_ms(1/e_status.frequency as u64);
        if uart.read_ready() {
            let mut byte: u8;
            
            if uart.read(&mut single).ok().unwrap() > 0 {
                let byte = single[0];
                    if byte == b'\n' {
                        let cmd = core::str::from_utf8(&buf[..pos]).unwrap_or("");
                        match cmd.trim() {
                        "DUMP" => print_csv(&readings),
                        "CLEAR" => readings.clear(),
                        _ => println!("Unknown command: {}", cmd),
                    }
                }
            };

        if e_status.recording {
            match scd41.data_ready_status() {
                Ok(status) => {
                    s = status;
                }
                Err(e) => info!("Data ready error: {}", Debug2Format(&e)),
            }

            if s {
                s = false;
                match scd41.measurement() {    
                    Ok(data) => {
                        readings.push(Reading {co2: data.co2, temperature: data.temperature, humidity: data.humidity, time: Instant::now().duration_since_epoch().as_millis() - e_status.start_time_ms});            
                    }
                    Err(e) => info!("Measurment Error: {}", Debug2Format(&e)),
                }
            };
        };
        }
    }
}
