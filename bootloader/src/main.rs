#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#[macro_use]
extern crate alloc;
use core::{mem, ptr, u8};

use log::info;
use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};
use uefi::proto::media::file::FileInfo;
use uefi::proto::media::file::{File, FileAttribute, FileType::Regular};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::MemoryDescriptor;
use uefi::{data_types::*, prelude::*, proto::loaded_image::LoadedImage};

#[repr(C)]
struct GopRes {
    x: u64,
    y: u64,
}
#[repr(C)]
pub struct GopInfo {
    pointer: *mut u8,
    size: u64,
    resolution: GopRes,
    stride: u64,
}

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).expect_success("Failed to initialize utilities");
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
    let bs = st.boot_services();

    let gop_raw = bs.locate_protocol::<GraphicsOutput>().unwrap().unwrap();
    let gop = unsafe { gop_raw.get().as_mut().unwrap() };
    let gop_2 = unsafe { gop_raw.get().as_mut().unwrap() };
    let modes = gop.modes();
    let mut gop_mode: Option<Mode> = None;
    for mode in modes {
        let mode = mode.unwrap();
        let info = mode.info();
        let res = info.resolution();
        if info.pixel_format() == PixelFormat::Bgr && res.0 == 1600 && res.1 == 900 {
            gop_2.set_mode(&mode).unwrap().unwrap();
            gop_mode = Some(mode);
            info!("Set GOP mode");
            break;
        }
    }
    let gop_mode = gop_mode.unwrap();
    let loaded_image = bs.handle_protocol::<LoadedImage>(image).unwrap().unwrap();
    let loaded_image = unsafe { loaded_image.get().as_ref().unwrap() };
    let fs = bs
        .handle_protocol::<SimpleFileSystem>(loaded_image.device())
        .unwrap()
        .unwrap();
    let fs = unsafe { fs.get().as_mut().unwrap() };
    let mut volume = fs.open_volume().unwrap().unwrap();
    let kernel_handle = volume
        .open(
            "kernel.elf",
            uefi::proto::media::file::FileMode::Read,
            FileAttribute::READ_ONLY,
        )
        .expect("Failed to load kernel")
        .unwrap();
    let kernel = if let Regular(mut kernel_file) = kernel_handle.into_type().unwrap().unwrap() {
        let mut info_buf = vec![0; 102];
        let kernel_size = kernel_file
            .get_info::<FileInfo>(&mut info_buf)
            .unwrap()
            .unwrap()
            .file_size();
        let mut out_buffer = vec![0; kernel_size as usize];
        kernel_file.read(&mut out_buffer).unwrap().unwrap();
        out_buffer
    } else {
        panic!("Kernel file is a directory");
    };

    info!("Copying Kernel...");
    unsafe {
        let entry = *(kernel.as_ptr().offset(0x18) as *const u64);
        let program_headers_offset = *(kernel.as_ptr().offset(0x20) as *const u64);
        let program_headers = kernel.as_ptr().add(program_headers_offset as usize);
        let entry_size = *(kernel.as_ptr().offset(0x36) as *const u16);
        let entry_count = *(kernel.as_ptr().offset(0x38) as *const u16);
        info!("Entry size: {}, Entry count: {}", entry_size, entry_count);
        for i in 0..entry_count {
            let entry = program_headers.add((i * entry_size).into());
            let segment_type = *(entry as *const u32);
            info!("Segment type: {}", segment_type);
            if segment_type == 1 {
                let data_offset = *(entry.offset(0x8) as *const u64);
                let mem_addr = *(entry.offset(0x10) as *const u64);
                let size_file = *(entry.offset(0x20) as *const u64);
                let size_mem = *(entry.offset(0x28) as *const u64);
                info!(
                    "Writing segment of size {:X} from {:X} to {:X}",
                    size_mem, data_offset, mem_addr
                );
                ptr::write_bytes(mem_addr as *mut u8, 0, size_mem as usize);
                ptr::copy(
                    kernel.as_ptr().add(data_offset as usize),
                    mem_addr as *mut u8,
                    size_file as usize,
                );
            }
        }
        info!("FB size: {}", gop.frame_buffer().size());
        info!("Exiting boot services...");
        let max_mmap_size =
            st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
        let mut mmap_buf = vec![0; max_mmap_size].into_boxed_slice();
        st.exit_boot_services(image, &mut mmap_buf)
            .expect_success("Failed to exit boot services");
        info!("Launching Kernel at {:X}", entry);
        let entry_fn: extern "sysv64" fn(GopInfo) -> ! = mem::transmute(entry);
        entry_fn(GopInfo {
            pointer: gop.frame_buffer().as_mut_ptr(),
            size: (gop.frame_buffer().size() / 4) as u64,
            resolution: GopRes {
                x: gop_mode.info().resolution().0 as u64,
                y: gop_mode.info().resolution().1 as u64,
            },
            stride: gop_mode.info().stride() as u64,
        });
    }
}
