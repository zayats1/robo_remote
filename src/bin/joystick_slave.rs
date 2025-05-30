//! ESP-NOW Example
//!
//! Broadcasts, receives and sends messages via esp-now

//% FEATURES: esp-wifi esp-wifi/esp-now esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use core::str::{self};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    rng::Rng,
    timer::timg::TimerGroup,
    uart::{self, Uart},
};
use esp_println::println;
use esp_wifi::{EspWifiController, esp_now::PeerInfo, init};
use robo_remote::{self as _, mk_static};

const THE_ADDRESS: [u8; 6] = [0x54u8, 0x32, 0x04, 0x32, 0xf2, 0xb8];
const WIFI_CHANNEL: u8 = 3;

// so MCU shouldn't halt
const INTERVAL: Duration = Duration::from_nanos(1);

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
    let mut esp_now = esp_wifi::esp_now::EspNow::new(esp_wifi_ctrl, wifi).unwrap();
    println!("esp-now version {}", esp_now.version().unwrap());
    esp_now.set_channel(WIFI_CHANNEL).unwrap();
    use esp_hal::timer::systimer::SystemTimer;
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let mut uart1 = Uart::new(
        peripherals.UART1,
        uart::Config::default().with_baudrate(115200),
    )
    .unwrap()
    .with_rx(peripherals.GPIO1)
    .with_tx(peripherals.GPIO2)
    .into_async();

    loop {
        let r = esp_now.receive_async().await;
        if r.info.dst_address == THE_ADDRESS {
            let data = r.data();
            let rec = str::from_utf8(data).unwrap_or("Data is not received properly");
            uart1.write_async(data).await.unwrap();
            println!("Received {}", rec);
            if !esp_now.peer_exists(&r.info.src_address) {
                esp_now
                    .add_peer(PeerInfo {
                        peer_address: r.info.src_address,
                        lmk: None,
                        channel: None,
                        encrypt: false,
                    })
                    .unwrap();
            }
        }
        Timer::after(INTERVAL).await;
    }
}
