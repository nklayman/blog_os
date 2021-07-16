#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
mod utils;
use utils::framebuffer::{set_framebuffer, FramebufferInfo};
use utils::interrupts;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start(fb_info: FramebufferInfo) -> ! {
    // Initialize interrupts
    interrupts::init();
    // The bootloader passes in the fb_info struct when starting the kernel
    // We use it to set the global framebuffer so that print! and println! work
    set_framebuffer(fb_info);
    // unsafe { asm!("ud2") };
    unsafe { *(0xd25235dbeaf as *mut u64) = 42 };

    println!("Hello, {}", "World!");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
