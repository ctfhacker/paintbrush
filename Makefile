check:
	# Clippy checks
	RUST_BACKTRACE=full cargo clippy -- \
				   --allow clippy::print_with_newline \
				   --allow clippy::write_with_newline \
				   --deny  missing_docs \
				   --deny  clippy::missing_docs_in_private_items \
				   --deny  clippy::pedantic \
				   --allow clippy::struct_excessive_bools \
				   --allow clippy::redundant_field_names \
				   --allow clippy::must_use_candidate

	# Documentation build regardless of arch
	cargo doc --no-deps -Zrustdoc-map

aarch64: check
	# Use the custom target.spec.json for now
	cargo build --release --target .cargo/aarch64-unknown-uefi.json

	# Ensure qemu_fs is created
	mkdir qemu_fs && echo 'paintbrush.efi' > qemu_fs/startup.nsh || true

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

x86: 
	cargo build --release
	/bin/cp target/x86_64-unknown-uefi/release/paintbrush.efi ../snapchange/pxe/snapchange.boot
	make check

qemu_x86: check
	cargo build --release

	# Run the dev in qemu
	qemu-system-x86_64 \
		-m 4G \
		-smp 2 \
		-bios /usr/share/OVMF/OVMF_CODE.fd \
		-nographic \
		-device driver=e1000,netdev=n0 \
		-netdev user,id=n0,tftp=target/x86_64-unknown-uefi/release,bootfile=paintbrush.efi
