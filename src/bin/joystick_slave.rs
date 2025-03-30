//! ESP-NOW Example
//!
//! Broadcasts, receives and sends messages via esp-now

//% FEATURES: esp-wifi esp-wifi/esp-now esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use core::str;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_wifi::{
    esp_now::{EspNowWifiInterface, PeerInfo},
    init, EspWifiController,
};
use robo_remote::{self as _, mk_static};


const THE_ADDRESS: [u8;6] = [0x54u8,0x32,0x04,0x32,0xf2,0xb8];

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
    controller.set_mode(esp_wifi::wifi::WifiMode::ApSta).unwrap();
    controller.start().unwrap();

    let mut esp_now = interfaces.esp_now;
 
    println!("esp-now version {}", esp_now.version().unwrap());

    use esp_hal::timer::systimer::SystemTimer;
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    loop {
        let r = esp_now.receive_async().await;
        println!(
            "Received {:?}",
            str::from_utf8(r.data()).unwrap_or("Data is not received properly")
        );
        if r.info.dst_address == THE_ADDRESS {
            if !esp_now.peer_exists(&r.info.src_address) {
                esp_now
                    .add_peer(PeerInfo {peer_address:r.info.src_address,lmk:None,channel:None,encrypt:false, interface: EspNowWifiInterface::Ap}
                    )
                    .unwrap();
            }
        }
        Timer::after_millis(2).await;
    }
}
