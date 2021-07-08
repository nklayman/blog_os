#![no_std]
#![no_main]

mod utils;
use utils::framebuffer::*;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start(fb_info: FramebufferInfo) -> ! {
    set_framebuffer(fb_info);
    println!("Hello, {}", "name");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
