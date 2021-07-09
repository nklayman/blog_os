#![no_std]
#![no_main]

mod utils;
use utils::framebuffer::*;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start(fb_info: FramebufferInfo) -> ! {
    // The bootloader passes in the fb_info struct when starting the kernel
    // We use it to set the global framebuffer so that print! and println! work
    set_framebuffer(fb_info);
    println!("Hello, {}", "World!");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
