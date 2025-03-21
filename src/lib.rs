#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::dbg!("{?:}",info);
    loop {}
}