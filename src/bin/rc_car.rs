//! ESP-NOW Example
//!
//! Broadcasts, receives and sends messages via esp-now

//% FEATURES: esp-wifi esp-wifi/esp-now esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use core::str::{self};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    mcpwm::{McPwm, PeripheralClockConfig, operator::PwmPinConfig, timer::PwmWorkingMode},
    rng::Rng,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{EspWifiController, esp_now::PeerInfo, init};
use robo_remote::{
    self as _,
    drivers::motor::Motor,
    mk_static,
    protocol::{message, parser::parse},
};

const THE_ADDRESS: [u8; 6] = [0x54u8, 0x32, 0x04, 0x32, 0xf2, 0xb8];
const WIFI_CHANNEL: u8 = 3;

// so MCU shouldn't halt
const INTERVAL: Duration = Duration::from_nanos(1);

const TIMEOUT: Duration = Duration::from_secs(5);

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

    // initialize peripheral
    let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(32)).unwrap();
    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);

    // connect operator0 to timer0
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    mcpwm.operator1.set_timer(&mcpwm.timer1);

    // connect operator0 to pin
    let pwm_pins = mcpwm.operator0.with_pins(
        peripherals.GPIO9,
        PwmPinConfig::UP_ACTIVE_HIGH,
        peripherals.GPIO8,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    let mut left_motor = Motor::new(pwm_pins.0, pwm_pins.1);
    // start timer with timestamp values in the range of 0..=99 and a frequency
    // of 20 kHz

    let pwm_pins2 = mcpwm.operator1.with_pins(
        peripherals.GPIO7,
        PwmPinConfig::UP_ACTIVE_HIGH,
        peripherals.GPIO6,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    let mut right_motor = Motor::new(pwm_pins2.0, pwm_pins2.1);

    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, Rate::from_khz(20))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);

   

    // TODO: timeout
    let mut receive_data = async move || {
        let rec = esp_now.receive_async().await;
        let received = if rec.info.dst_address == THE_ADDRESS {
            let data = rec.data();
            let received = str::from_utf8(data);

            println!("Received {:?}", rec);
            if !esp_now.peer_exists(&rec.info.src_address) {
                esp_now
                    .add_peer(PeerInfo {
                        peer_address: rec.info.src_address,
                        lmk: None,
                        channel: None,
                        encrypt: false,
                    })
                    .unwrap();
            }

            Some(received.ok())
        } else {
            println!("Receiving error");
            None
        };

        return if let Some(received) = received.flatten() {
            match parse(received) {
                Ok(message) => Some(message),
                Err(err) => {
                    println!("{}", err);
                    None
                }
            }
        } else {
            None
        };
    };

    loop {
        let res = select(receive_data(), Timer::after(TIMEOUT).into_future()).await;

        let received = match res {
            Either::First(rec) => rec,
            Either::Second(_) => {
                println!("Disconnected");
                left_motor.stop();
                right_motor.stop();
                None
            }
        };

        if let Some(message) = received {
            match message {
                // Todo: make speed stable
                message::Message::LeftSpeed(speed) => left_motor.run(speed as i16),
                message::Message::RightSpeed(speed) => right_motor.run(speed as i16),
                message::Message::Stop => {
                    left_motor.stop();
                    right_motor.stop();
                }
            }
        }

        Timer::after(INTERVAL).await;
    }
}
