#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
mod utils;
use defmt::Debug2Format;
use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{delay::Delay, main, time::Rate};
use esp_radio::ble::controller::BleConnector;
use scd4x::Scd4x;
use {esp_backtrace as _, esp_println as _};
use utils::_delay_ms;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.2.0
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    _delay_ms(1000);

    let led_gpio = peripherals.GPIO2;
    let sda_gpio = peripherals.GPIO22;
    let scl_gpio = peripherals.GPIO21;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let _connector = BleConnector::new(&radio_init, peripherals.BT, Default::default());

    // Set GPIO2 as an output, and set its state to high initially.
    let mut _led = Output::new(led_gpio, Level::Low, OutputConfig::default());

    //create i2c
    let slow_clock = Config::default().with_frequency(Rate::from_khz(10));
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

    loop {
        _delay_ms(100u64); 
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
                    info!("CO2: {}", data.co2);
                    info!("Temp: {}", data.temperature);
                    info!("Humidity: {}", data.humidity)
                }
                Err(e) => info!("Measurment Error: {}", Debug2Format(&e)),
            }
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
