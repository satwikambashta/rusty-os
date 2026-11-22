#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use rusty_os_kernel::console::Console;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial();
    let _ = writeln!(serial, "Rusty OS: kernel entry reached");

    let Some(framebuffer) = boot_info.framebuffer.as_mut() else {
        let _ = writeln!(serial, "Rusty OS: no framebuffer available");
        halt_loop()
    };

    let info = framebuffer.info();
    let Some(mut console) = Console::new(framebuffer.buffer_mut(), info) else {
        let _ = writeln!(serial, "Rusty OS: unsupported framebuffer configuration");
        halt_loop()
    };

    let _ = writeln!(console, "Rusty OS");
    let _ = writeln!(console, "Kernel entered successfully.");
    let _ = writeln!(console, "Framebuffer: {} x {}", info.width, info.height);
    let _ = writeln!(console, "Serial console: COM1");
    halt_loop()
}

fn serial() -> Uart16550Tty<PioBackend> {
    unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) }
        .expect("failed to initialize COM1 serial port")
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if let Ok(mut serial) = unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) } {
        let _ = writeln!(serial, "Rusty OS kernel panic: {info}");
    }

    halt_loop()
}

fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}
