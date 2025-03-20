#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::clock::CpuClock;
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::ADC1;
use esp_hal::timer::systimer::SystemTimer;
// use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use panic_halt as _;

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // generator version: 0.3.1
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // let timer1 = TimerGroup::new(peripherals.TIMG0);
    let analog_pin = peripherals.GPIO1;
    let mut adc1_config = AdcConfig::new();

    let mut pin = adc1_config.enable_pin(analog_pin, Attenuation::_0dB);
    let mut adc1 = Adc::new(unsafe {peripherals.ADC1.clone_unchecked()}, adc1_config).into_async();

    let analog_pin2 = peripherals.GPIO2;
    let mut adc12_config = AdcConfig::new();
    let mut pin2 = adc12_config.enable_pin(analog_pin2, Attenuation::_0dB);
    let mut adc12 = Adc::new(peripherals.ADC1, adc12_config).into_async();
    loop {
        let pin_value = adc1.read_oneshot(&mut pin).await;

        println!("X value: {}", pin_value.saturating_sub(2034));
        let pin2_value = adc12.read_oneshot(&mut pin2).await;

        println!("Y value: {}", pin2_value.saturating_sub(2034));

        Timer::after(Duration::from_millis(500)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
