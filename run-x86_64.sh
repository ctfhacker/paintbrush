# Use the custom target.spec.json for now
cargo build --release

# Run the dev in qemu
qemu-system-x86_64 \
    -m 4G \
    -smp 2 \
    -bios /usr/share/OVMF/OVMF_CODE.fd \
    -nographic \
    -device driver=e1000,netdev=n0 \
    -netdev user,id=n0,tftp=target/x86_64-unknown-uefi/release,bootfile=paintbrush.efi
