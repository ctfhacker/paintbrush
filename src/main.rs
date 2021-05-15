//! Build script for the kernel and bootloader
//!
//! Bootloader: [`Docs`](../../../bootloader/target/x86_64-unknown-uefi/doc/bootloader/index.html)
//!
//! Kernel: [`Docs`](../../../kernel/target/x86_64-pc-windows-msvc/doc/kernel/index.html)
use anyhow::Result;

use std::process::Command;
use std::path::Path;

/// Copy the resulting binaries to the output directory
pub fn copy_binaries() -> Result<()> {

    Ok(())
}

pub fn build_x86() -> Result<()> {
    // Output directory to drop all the finished binaries
    let output_dir = "build";

    // Create the output directory
    std::fs::create_dir_all(output_dir)?;

    // Build the bootloader
    if !Command::new("cargo")
            .current_dir("bootloader")
            .args(&["build", "--release"])
            .status()?
            .success() {
        panic!("Failed to build bootloader");
    }

    // Create the output path for the x86 bootloader
    let bootloader_path = Path::new("bootloader")
        .join("target")
        .join("x86_64-unknown-uefi")
        .join("release")
        .join("bootloader.efi");

    // Copy the bootloader into the output directory
    std::fs::copy(&bootloader_path, Path::new(output_dir).join("paintbrush_x86.boot"))?;
    std::fs::copy(bootloader_path, Path::new(output_dir).join("snapchange.boot"))?;

    // Build the kernel
    if !Command::new("cargo")
            .current_dir("kernel")
            .args(&["build", "--release"])
            .status()?
            .success() {
        panic!("Failed to build kernel");
    }

    // Create the output path for the x86 kernel
    let bootloader_path = Path::new("kernel")
        .join("target")
        .join("x86_64-pc-windows-msvc")
        .join("release")
        .join("kernel.exe");

    // Copy the bootloader into the output directory
    std::fs::copy(&bootloader_path, Path::new(output_dir).join("paintbrush_x86.kernel"))?;
    std::fs::copy(bootloader_path, Path::new(output_dir).join("snapchange.kernel"))?;

    Ok(())
}

pub fn build_arm() -> Result<()> {
    // Output directory to drop all the finished binaries
    let output_dir = "build";

    // Create the output directory
    std::fs::create_dir_all(output_dir)?;

    // Build the bootloader
    if !Command::new("cargo")
            .current_dir("bootloader")
            .args(&["build", "--release", "--target", ".cargo/aarch64-unknown-uefi.json"])
            .status()?
            .success() {
        panic!("Failed to build bootloader");
    }

    // Create the output path for the arm bootloader
    let bootloader_path = Path::new("bootloader")
        .join("target")
        .join("aarch64-unknown-uefi")
        .join("release")
        .join("bootloader.efi");

    // Copy the bootloader into the output directory
    std::fs::copy(bootloader_path, Path::new(output_dir).join("paintbrush_arm.boot"))?;

    // Build the kernel
    if !Command::new("cargo")
            .current_dir("kernel")
            .args(&["build", "--release", "--target", ".cargo/aarch64-unknown-uefi.json"])
            .status()?
            .success() {
        panic!("Failed to build kernel");
    }

    // Create the output path for the arm kernel
    let bootloader_path = Path::new("kernel")
        .join("target")
        .join("aarch64-unknown-uefi")
        .join("release")
        .join("kernel.efi");

    // Copy the bootloader into the output directory
    std::fs::copy(bootloader_path, Path::new(output_dir).join("paintbrush_arm.kernel"))?;

    Ok(())
}


/// Build the bootloader and kernel
pub fn build_binaries() -> Result<()> {
    build_x86()?;
    // build_arm()?;
    Ok(())
}

/// Run clippy on the bootloader and kernel
pub fn clippy_check() -> Result<()> {
    for project in &["./bootloader", "./kernel"] {
        // Run clippy on the bootloader
        Command::new("cargo")
            .current_dir(project)
            .arg("clippy").arg("--")
            .arg("--allow").arg("clippy::print_with_newline")
            .arg("--allow").arg("clippy::write_with_newline")
            .arg("--deny").arg("missing_docs")
            .arg("--deny").arg("clippy::missing_docs_in_private_items")
            .arg("--allow").arg("clippy::struct_excessive_bools")
            .arg("--allow").arg("clippy::redundant_field_names")
            .arg("--allow").arg("clippy::must_use_candidate")
            .status()?;
    }

    Ok(())
}

/// Build docs for the bootloader and kernel
pub fn build_docs() -> Result<()> {
    for project in &["./", "./bootloader", "./kernel"] {
        Command::new("cargo")
            .current_dir(project)
            .arg("doc")
            .status()?;
    }

    Ok(())
}

/// Build script for the bootloader and kernel
pub fn main() -> Result<()> {
    build_binaries()?;
    clippy_check()?;
    copy_binaries()?;
    build_docs()?;
    Ok(())
}
