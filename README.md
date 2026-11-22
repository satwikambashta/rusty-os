# Rusty OS

Rusty OS is an x86_64 hobby kernel built with Rust 2024 and the current
bootloader API. It follows the learning path of Philipp Oppermann's
[Writing an OS in Rust](https://os.phil-opp.com/) series while using current
tooling and kernel interfaces where practical.

## Current status

The kernel uses the `bootloader_api` interface to display a text console on the
bootloader-provided framebuffer and writes boot and panic messages to the COM1
serial port. The framebuffer console supports RGB, BGR, grayscale, and common
packed pixel formats, with line wrapping and scrolling. The root package builds
a BIOS boot image and `cargo run` launches it in QEMU.

## Build

The pinned toolchain is nightly because Cargo artifact dependencies and the
bootloader's image builder require nightly features and LLVM tools. Install
QEMU if needed (on Ubuntu, run `sudo apt install qemu-system-x86`), then run:

```sh
cargo build
cargo test --workspace
cargo run
```

The bootable image is written under Cargo's `target` build directory. Pass
additional arguments to QEMU after `cargo run --`, for example
`cargo run -- -m 256M`.

## Roadmap

- Set up framebuffer output, CPU exception handling, and hardware interrupts.
- Implement physical and virtual memory management and a kernel heap.
- Add task scheduling, async support, and device drivers.

The implementation is being developed incrementally; memory management and
multitasking from the reference series are not implemented yet.
