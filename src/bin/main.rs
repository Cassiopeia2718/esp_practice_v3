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

use defmt::{info, Debug2Format};

use esp_hal::{gpio::{Level, Output, OutputConfig}, i2c::master::{Config as i2c_Config, I2c}, uart::{Uart, Config as uart_Config}};
use esp_hal::{clock::CpuClock, delay::Delay, main, time::{Rate, Instant}};

use scd4x::Scd4x;

use {esp_backtrace as _, esp_println::println};
use esp_practice_v3::{_delay_ms, send_data, read_command, Reading, Status};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    /*HARDWARE SETUP*/ 
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        //GPIO setup
        let led_gpio = peripherals.GPIO2;
        //TODO: Measurment LED
        //TODO: Power LED
        //TODO: Connected to computer LED
        //TODO: Power Button

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
    
    /*SOFTWARE SETUP & VERIFICATION */
        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    
    

        match scd41.start_periodic_measurement() {
            Ok(_) => info!("measurments started"),
            Err(e) => info!("Measurment faild: {}", Debug2Format(&e)),
        }

        let mut s = false;

        let mut p = 0;

        let mut e_status = Status {
            recording: false,
            frequency: 1,
            start_time_ms: Instant::now().duration_since_epoch().as_millis(),
        };

        let mut uart = Uart::new(peripherals.UART0, uart_Config::default())
            .unwrap()
            .with_rx(peripherals.GPIO3)
            .with_tx(peripherals.GPIO1);

        

        let mut readings: Vec<Reading> = Vec::new();

    /* Loop to run until poweroff*/
    loop {
        _delay_ms(1/e_status.frequency as u64);
        let r = read_command(&mut uart);
        match r {
            0 => {},
            1 => e_status = Status {
                recording: true,
                frequency: e_status.frequency,
                start_time_ms: Instant::now().duration_since_epoch().as_millis(),
                },
            2 => e_status.recording = false,
            3 => send_data(&mut uart, &mut p, &readings),
            4..=255 => e_status.frequency = 1/r as u16,
        }

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
