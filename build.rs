use std::path::PathBuf;

fn main() {
    let kernel = PathBuf::from(
        std::env::var_os("CARGO_BIN_FILE_RUSTY_OS_KERNEL_rusty-os-kernel")
            .expect("kernel artifact dependency was not built"),
    );
    let image = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is not set"))
        .join("rusty-os-bios.img");

    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&image)
        .expect("failed to create BIOS boot image");

    println!("cargo:rustc-env=BIOS_IMAGE={}", image.display());
}
