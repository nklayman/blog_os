#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#[macro_use]
extern crate alloc;
use core::{mem, ptr};

use log::info;
use uefi::proto::media::file::{File, FileAttribute, FileType::Regular};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::MemoryDescriptor;
use uefi::{data_types::*, prelude::*, proto::loaded_image::LoadedImage};

#[repr(packed)]
pub struct KernelArgs {
    kernel_base: u64,
    kernel_size: u64,
    stack_base: u64,
    stack_size: u64,
    env_base: u64,
    // acpi_rsdps_base: u64,
    // acpi_rsdps_size: u64,
}

static PHYSICAL_OFFSET: u64 = 0xFFFF800000000000;

static KERNEL_PHYSICAL: u64 = 0x100000;
static mut KERNEL_SIZE: u64 = 0;
static mut KERNEL_ENTRY: u64 = 0;

static STACK_PHYSICAL: u64 = 0x80000;
static STACK_VIRTUAL: u64 = STACK_PHYSICAL + PHYSICAL_OFFSET;
static STACK_SIZE: u64 = 0x1F000;

static mut ENV_SIZE: u64 = 0x0;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).expect_success("Failed to initialize utilities");
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
    let bs = st.boot_services();
    let loaded_image = bs.handle_protocol::<LoadedImage>(image).unwrap().unwrap();
    let loaded_image = unsafe { loaded_image.get().as_ref().unwrap() };
    let fs = bs
        .handle_protocol::<SimpleFileSystem>(loaded_image.device())
        .unwrap()
        .unwrap();
    let fs = unsafe { fs.get().as_mut().unwrap() };
    let mut volume = fs.open_volume().unwrap().unwrap();
    let mut thing = vec![0; 2000];
    let mut out_buffer = vec![0; 2000000];
    info!("{:?}", out_buffer.as_ptr());
    info!("{}", thing[0]);
    let kernel = volume
        .open(
            "kernel.elf",
            uefi::proto::media::file::FileMode::Read,
            FileAttribute::READ_ONLY,
        )
        .expect("Failed to load kernel")
        .unwrap();
    if let Regular(mut kernel_file) = kernel.into_type().unwrap().unwrap() {
        info!(
            "read into buffer: {} bytes",
            kernel_file.read(&mut out_buffer).unwrap().unwrap()
        );
    }
    // let mut mmap_buf = vec![0; bs.memory_map_size()];
    // info!("mmap size: {}", mmap_buf.len());
    info!("Copying Kernel...");
    unsafe {
        KERNEL_SIZE = out_buffer.len() as u64;
        info!("Size: {}", KERNEL_SIZE);
        KERNEL_ENTRY = *(out_buffer.as_ptr().offset(0x18) as *const u64);
        info!("Entry: {:X}", KERNEL_ENTRY);
        ptr::copy(
            out_buffer.as_ptr(),
            KERNEL_PHYSICAL as *mut u8,
            out_buffer.len(),
        );
    }
    let env = "";
    info!("Copying Environment...");
    unsafe {
        ENV_SIZE = env.len() as u64;
        info!("Size: {}", ENV_SIZE);
        info!("Data: {}", env);
        ptr::copy(env.as_ptr(), STACK_PHYSICAL as *mut u8, env.len());
    }

    // info!("Exiting boot services...");
    // let max_mmap_size =
    //     st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    // let mut mmap_buf = vec![0; max_mmap_size].into_boxed_slice();
    // st.exit_boot_services(image, &mut mmap_buf)
    //     .expect_success("Failed to exit boot services");
    info!("Launching kernel...");
    unsafe {
        let entry_fn: extern "sysv64" fn() -> i64 = mem::transmute(KERNEL_ENTRY);
        // info!("{:?}", entry_fn);
        info!("{}", entry_fn());
        info!("{}", out_buffer.len());
        // enter();
    }
    // unsafe {
    //     enter();
    // }

    // info!("{:?}", out_buffer);
    // let h = st
    //     .boot_services()
    //     .get_image_file_system(image)
    //     .expect("Failed to get image")
    //     .unwrap();

    loop {}
}

unsafe fn enter() -> ! {
    let args = KernelArgs {
        kernel_base: KERNEL_PHYSICAL,
        kernel_size: KERNEL_SIZE,
        stack_base: STACK_VIRTUAL,
        stack_size: STACK_SIZE,
        env_base: STACK_VIRTUAL,
        // env_size: ENV_SIZE,
        // acpi_rsdps_base: RSDPS_AREA
        //     .as_ref()
        //     .map(Vec::as_ptr)
        //     .unwrap_or(core::ptr::null()) as usize as u64
        //     + PHYSICAL_OFFSET,
        // acpi_rsdps_size: RSDPS_AREA.as_ref().map(Vec::len).unwrap_or(0) as u64,
    };

    info!("running kernel");
    let entry_fn: extern "sysv64" fn() -> i64 = mem::transmute(KERNEL_ENTRY);
    info!("out: {}", entry_fn());
    loop {}
}
