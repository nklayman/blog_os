#[repr(packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

// impl GdtDescriptor {
//     pub fn new(size: u16, offset: u64) -> Self {
//         Self { size, offset }
//     }
// }

struct SegmentDescriptor {
  
}

struct Gdt([SegmentDescriptor; 16]);