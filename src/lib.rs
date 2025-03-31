#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::dbg!("{?:}", info);
    loop {}
}
// from esp32 examples
// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

// from arduino

pub trait Map {
    fn map(self, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32;
}

impl Map for f32 {
    #[inline]
    fn map(self, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        (self - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }
}
