use conquer_once::spin::OnceCell;
use core::{fmt, ptr};
use spin::Mutex;

static FONT: &[u8] = include_bytes!("../font.psf");
#[repr(packed)]
#[allow(dead_code)]
struct PsfHeader {
    magic: u32,         /* magic bytes to identify PSF */
    version: u32,       /* zero */
    headersize: u32,    /* offset of bitmaps in file, 32 */
    flags: u32,         /* 0 if there's no unicode table */
    numglyph: u32,      /* number of glyphs */
    bytesperglyph: u32, /* size of each glyph */
    height: u32,        /* height in pixels */
    width: u32,         /* width in pixels */
}

#[repr(C)]
struct FramebufferRes {
    x: u64,
    y: u64,
}

/// This struct is passed to the kernel by the bootloader
/// and contains all the necessary information to use the GOP framebuffer
#[repr(C)]
pub struct FramebufferInfo {
    pointer: *mut u32,
    size: u64,
    resolution: FramebufferRes,
    stride: u64,
}

/// The FramebufferInfo struct can be turned into this, which has implementations for writing text
pub struct Framebuffer {
    info: FramebufferInfo,
    current_line: usize,
    current_col: usize,
}
unsafe impl Send for Framebuffer {}

pub static FRAMEBUFFER: OnceCell<Mutex<Framebuffer>> = OnceCell::uninit();

/// This should be called at the start of the kernel to set the global Framebuffer object
/// which is used by the print! and println! macros
pub fn set_framebuffer(fb_info: FramebufferInfo) {
    let fb = Framebuffer::new(fb_info);
    fb.clear();
    let fb = Mutex::new(fb);
    FRAMEBUFFER.init_once(move || fb)
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::utils::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Just a wrapper to call the global framebuffer write_fmt
/// Used by the print! and println! macros
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    FRAMEBUFFER.get().unwrap().lock().write_fmt(args).unwrap();
}

impl Framebuffer {
    /// Create a Framebuffer from the info passed to kernel by bootloader
    pub fn new(fb_info: FramebufferInfo) -> Self {
        Self {
            info: fb_info,
            current_line: 0,
            current_col: 0,
        }
    }
    /// Print a string to the current line and col positions, auto wraps
    pub fn print(&mut self, text: &str) {
        let (header_size, bytes_per_glyph, height, width) = get_font_info();
        for c in text.bytes() {
            // If text overflows line or if character is a newline, move down a line
            if c == b'\n' || self.current_col as u64 + width > self.info.stride {
                self.current_line += 1;
                self.current_col = 0;
                // Don't write char if it's a newline
                if c == b'\n' {
                    continue;
                }
            }
            // This is the location of the first byte of the glyph
            let char_pos = header_size + bytes_per_glyph * c as usize;
            // Draw the font line by line, one pixel at a time
            for y in 0..height {
                // Each byte corresponds to one line (this won't support wider fonts)
                let line_data = FONT[char_pos + y as usize];
                for x in 0..width {
                    // Convert the byte to binary, and a 1 means draw a pixel, a 0 means don't
                    // at each x position
                    // This bitwise operation returns true if there is a 1 at the x coordinate
                    if line_data & (1 << (width - 1 - x)) != 0 {
                        self.draw_point(
                            x + self.current_col as u64,
                            y + self.current_line as u64 * height,
                        )
                    }
                }
            }
            // Move the col position over by the width + 1 (for spacing between letters)
            self.current_col += (width + 1) as usize;
        }
    }
    pub fn _draw_rect(&self, x: u64, y: u64, width: u64, height: u64) {
        let mut cursor = (x, y);
        loop {
            self.draw_point(cursor.0, cursor.1);
            cursor.0 += 1;
            if cursor.0 == (x + width) {
                if cursor.1 == y + height {
                    break;
                }
                cursor.0 = x;
                cursor.1 += 1;
            }
        }
    }

    /// Draws a point at the specified x/y location, no support for colors (yet)
    pub fn draw_point(&self, x: u64, y: u64) {
        let fb = &self.info;
        let offset = x + y * fb.stride;
        if offset < fb.size {
            unsafe {
                ptr::write(fb.pointer.add(offset as usize), 255);
            }
        }
    }

    /// Clears out the framebuffer by writing it all to 0s
    pub fn clear(&self) {
        unsafe {
            ptr::write_bytes(self.info.pointer, 0, self.info.size as usize);
        }
    }
}

/// Gets info about the fonts from the PSF header
/// Supports PSF v1 and v2
fn get_font_info() -> (usize, usize, u64, u64) {
    let (header_size, bytes_per_glyph, height, width) = if FONT[0..2] == [0x36, 0x04] {
        // PSFv1
        (4, FONT[3] as usize, FONT[3] as u64, 8 as u64)
    } else if FONT[0..4] == [0x72, 0xb5, 0x4a, 0x86] {
        // PSFv2
        let header: &PsfHeader = unsafe { &*(FONT.as_ptr() as *const PsfHeader) };
        (
            header.headersize as usize,
            header.bytesperglyph as usize,
            header.height as u64,
            header.width as u64,
        )
    } else {
        // Unknown PSF version
        panic!("This message is useless because we can't print")
    };
    (header_size, bytes_per_glyph, height, width)
}

impl fmt::Write for Framebuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}
