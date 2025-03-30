//! ESP-NOW Example
//!
//! Broadcasts, receives and sends messages via esp-now

//% FEATURES: esp-wifi esp-wifi/esp-now esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    peripheral::Peripheral,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{
    EspWifiController,
    esp_now::{EspNowWifiInterface, PeerInfo},
    init,
};
use heapless::String;
use robo_remote::{self as _, mk_static};

const ADC_SHIFT: u16 = 2144; // to obtain zero at the minimum of a joystick range

const PEER_ADDRESS: [u8; 6] = [0x54, 0x32, 0x04, 0x32, 0xf2, 0xb8];

// TODO: master address
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(
            timg0.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );

    let wifi = peripherals.WIFI;
    let (mut controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();
    controller
        .set_mode(esp_wifi::wifi::WifiMode::ApSta)
        .unwrap();
    controller.start().unwrap();

    let mut esp_now = interfaces.esp_now;

    println!("esp-now version {}", esp_now.version().unwrap());

    use esp_hal::timer::systimer::SystemTimer;
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let mut ticker = Ticker::every(Duration::from_millis(500)); // todo: make faster
    let mut data: String<64> = String::new();

    let analog_pin = peripherals.GPIO1;
    let mut adc1_config = AdcConfig::new();

    let mut pin = adc1_config.enable_pin(analog_pin, Attenuation::_11dB);

    let adc = peripherals.ADC1;
    let mut adc1 = Adc::new(unsafe { adc.clone_unchecked() }, adc1_config).into_async();

    let mut adc12_config = AdcConfig::new();
    let analog_pin2 = peripherals.GPIO2;

    let mut pin2 = adc12_config.enable_pin(analog_pin2, Attenuation::_11dB);
    let mut adc12 = Adc::new(adc, adc12_config).into_async();

    loop {
        let x = adc1.read_oneshot(&mut pin).await.saturating_sub(ADC_SHIFT);
        println!("X value: {}", x);

        if !esp_now.peer_exists(&PEER_ADDRESS) {
            esp_now
                .add_peer(PeerInfo {
                    peer_address: PEER_ADDRESS,
                    lmk: None,
                    channel: None,
                    encrypt: false,
                    interface: EspNowWifiInterface::Ap,
                })
                .unwrap();
        }

        let y = adc12
            .read_oneshot(&mut pin2)
            .await
            .saturating_sub(ADC_SHIFT);
        println!("Y value: {}", y);
        Timer::after(Duration::from_millis(250)).await;

        data.clear();
        writeln!(&mut data, "X:{};Y:{};", x, y).unwrap(); // todo
        let status = esp_now.send_async(&PEER_ADDRESS, data.as_bytes()).await;
        println!("Send broadcast status: {:?}", status);

        ticker.next().await;
    }
}
