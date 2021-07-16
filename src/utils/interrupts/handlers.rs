use crate::println;
use bitflags::bitflags;
#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

macro_rules! exception_handler {
  ($name: ident) => {{
    #[naked]
    pub extern "C" fn wrapper() -> ! {
        unsafe {
            asm!(
              "push rax",
              "push rcx",
              "push rdx",
              "push rsi",
              "push rdi",
              "push r8",
              "push r9",
              "push r10",
              "push r11",
              "mov rdi, rsp",
              "sub rsp, 8", // align stack pointer to 16 bytes
              "call {}",
              "add rsp, 8",
              "pop r11",
              "pop r10",
              "pop r9",
              "pop r8",
              "pop rdi",
              "pop rsi",
              "pop rdx",
              "pop rcx",
              "pop rax",
              "iretq",
              sym $name,
              options(noreturn)
            )
        }
    }
    wrapper
  }};
}
pub(super) use exception_handler;

macro_rules! exception_handler_with_error_code {
  ($name: ident) => {{
      #[naked]
      extern "C" fn wrapper() -> ! {
          unsafe {
              asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "mov rsi, [rsp + 9*8]", // load error code into rsi
                "mov rdi, rsp",
                "add rdi, 10*8", // adjust for pushed registers
                "sub rsp, 8", // align stack pointer to 16 bytes
                "call {}",
                "add rsp, 8",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "add rsp, 8", // pop error code
                "iretq",
                sym $name,
                options(noreturn)
              );
          }
      }
      wrapper
  }}
}
pub(super) use exception_handler_with_error_code;

pub extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!(
        "\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
}

bitflags! {
  struct PageFaultErrorCode: u64 {
      const PROTECTION_VIOLATION = 1 << 0;
      const CAUSED_BY_WRITE = 1 << 1;
      const USER_MODE = 1 << 2;
      const MALFORMED_TABLE = 1 << 3;
      const INSTRUCTION_FETCH = 1 << 4;
  }
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    use x86_64::registers::control;
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\
      \nerror code: {:?}\n{:#?}",
        control::Cr2::read(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );
}

pub extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
