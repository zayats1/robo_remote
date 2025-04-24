#![no_std]
#![no_main]


use function_name::named;

use embassy_executor::Spawner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::
    clock::CpuClock
;

use esp_println::println;

use robo_remote::protocol::{
    message::Message,
    parser::{ParsingError, parse},
};

use esp_hal::timer::systimer::SystemTimer;


#[named]
fn parse_chassis_test() {
    println!("{}", function_name!());
    let message = "LSPEED:25.0;";
    let res = parse(message);
    assert_eq(res, Ok(Message::LeftSpeed(25.0)));

    let message = "RSPEED:25.08;";
    let res = parse(message);
    assert_eq(res, Ok(Message::RightSpeed(25.08)));

    let message = "STOP:;";
    let res = parse(message);
    assert_eq(res, Ok(Message::Stop));

    println!("PASSED");
}

#[named]
fn parse_value_error_test() {
    println!("{}", function_name!());
    let message = "LSPEED:;";
    let res = parse(message);
    assert_eq(res, Err(ParsingError::ValueCanNotBeParsed));
    println!("PASSED");
}
#[named]
fn parse_not_a_comand_error_test() {
    println!("{}", function_name!());
    let message = "STO:;";
    let res = parse(message);
    assert_eq(res, Err(ParsingError::NotAComand));

    println!("PASSED");
}

#[named]
fn parse_sepparator_error_test() {
    println!("{}", function_name!());
    let message = "LSPEED;";
    let res = parse(message);
    assert_eq(res, Err(ParsingError::NoSepparator));

    let message = "LSPEED:4";
    let res = parse(message);
    assert_eq(res, Err(ParsingError::NoSepparator));

    println!("PASSED");
}

fn assert_eq<U: PartialEq>(res: U, expected: U) {
    if res != expected {
        println!("FAILED");
        panic!();
    }
}

pub fn run_tests() {
    parse_chassis_test();
    parse_value_error_test();
    parse_not_a_comand_error_test();
    parse_sepparator_error_test();
    println!("All tests passed")
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    run_tests();
    loop {}
}
