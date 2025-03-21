#![no_std]

use esp_println::println;
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::dbg!("{?:}",info);
    loop {}
}