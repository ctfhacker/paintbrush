# Use the custom target.spec.json for now
cargo build --release --target .cargo/aarch64-unknown-uefi.json

# Ensure qemu_fs dir exists
mkdir qemu_fs 2>/dev/null

# Copy the kernel into the qemu_fs for testing
cp target/aarch64-unknown-uefi/release/paintbrush.efi qemu_fs/

# Run the dev in qemu
./aarch64_build/qemu/build-native/qemu-system-aarch64 \
    -m 8G \
    -smp 4 \
    -M sbsa-ref \
    -nographic \
    -drive file=fat:rw:./qemu_fs \
    -pflash ./aarch64_build/SBSA_FLASH0.fd \
    -pflash ./aarch64_build/SBSA_FLASH1.fd

