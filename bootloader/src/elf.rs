pub fn read_elf_header<'a>(file: &[u8]) -> &'a ELFHeader {
    unsafe { &*(file.as_ptr() as *const ELFHeader) }
}

#[repr(packed)]
pub struct ELFHeader {
    pub e_ident: ELFIdent,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(packed)]
pub struct ELFIdent {
    pub magic_num: u32,
    pub arch: u8,
    pub endianness: u8,
    pub header_version: u8,
    pub os_abi: u8,
    _pad: [u8; 8],
}
