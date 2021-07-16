use conquer_once::spin::Lazy;
mod idt;
use idt::*;
mod handlers;
use handlers::*;

pub static IDT: Lazy<Idt> = Lazy::new(|| {
    let mut idt = Idt::new();
    idt.set_handler(0, exception_handler!(divide_by_zero_handler));
    idt.set_handler(6, exception_handler!(invalid_opcode_handler));
    idt.set_handler(14, exception_handler_with_error_code!(page_fault_handler));
    idt.set_handler(8, exception_handler_with_error_code!(double_fault_handler));
    idt
});

pub fn init() {
    IDT.load();
}
