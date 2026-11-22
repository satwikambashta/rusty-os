use std::{
    env, io,
    process::{Command, ExitStatus, exit},
};

fn main() {
    match run() {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("{error}");
            exit(1);
        }
    }
}

fn run() -> io::Result<ExitStatus> {
    let image = env!("BIOS_IMAGE");
    Command::new("qemu-system-x86_64")
        .args([
            "-drive",
            &format!("format=raw,file={image}"),
            "-serial",
            "mon:stdio",
            "-display",
            "none",
            "-no-reboot",
        ])
        .args(env::args().skip(1))
        .status()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("failed to start QEMU; install qemu-system-x86_64 and try again: {error}"),
            )
        })
}
