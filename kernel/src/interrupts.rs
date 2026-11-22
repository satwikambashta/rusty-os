use core::fmt::Write;
use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt
});

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    fatal_exception("divide error", frame, None);
}

extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    let mut serial = super::serial();
    let _ = writeln!(serial, "Rusty OS: breakpoint exception");
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    fatal_exception("invalid opcode", frame, None);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    fatal_exception("general protection fault", frame, Some(error_code));
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address = x86_64::registers::control::Cr2::read();
    let mut serial = super::serial();
    let _ = writeln!(serial, "Rusty OS: page fault at {address:?}");
    let _ = writeln!(serial, "Error code: {error_code:?}");
    let _ = writeln!(serial, "Interrupt stack frame: {frame:#?}");
    super::halt_loop()
}

fn fatal_exception(name: &str, frame: InterruptStackFrame, error_code: Option<u64>) -> ! {
    let mut serial = super::serial();
    let _ = writeln!(serial, "Rusty OS: fatal CPU exception: {name}");
    if let Some(error_code) = error_code {
        let _ = writeln!(serial, "Error code: {error_code:#x}");
    }
    let _ = writeln!(serial, "Interrupt stack frame: {frame:#?}");
    super::halt_loop()
}
