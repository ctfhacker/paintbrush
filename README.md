# PaintBrush - Rust UEFI kernel

This kernel uses UEFI to boot and hot reload.

## Architecture

The bootloader is UEFI enabled and heavily leverages UEFI protocols. First, the
bootloader gets access to the memory map as seen by UEFI via [`uefi::get_memory_map`].
This returns a [`RangeSet`], which is used as a basic physical memory allocator. The
kernel is then downloaded from the same TFTP server that is serving the bootloader via
[`uefi::read_file`]. This kernel is parsed to extract the segments to properly map them
into memory. UEFI also enables multiprocessing functionality to boot individual cores.
The kernel is mapped into the bootloader's page table. This page table handed to us from
UEFI is an identity mapped page table. This page table is also the page table that will
be used to start the execution of each core, so the virtual address of the kernel
(`0xffff_8888_0000_0000` for example) must be mapped in the identity mapped page table as
well.

Once the kernel is mapped, the entry point from the kernel is also read from the pe
parser. This entry point is the procedure called during [`uefi::startup_this_ap`] to
start another core executing the downloaded kernel. 

The kernel entry point is given one argument, the physical address of a [`CoreArg`] 
struct. This [`CoreArg`] gives the bootloader the ability to pass information to the
kernel that is necessary for its execution. For example, the following are a few key
items passed to the kernel:

* New page table address: Each core will execute in its own page table. This page table
  will be setup by the bootloader and is expected to be used immediately when the kernel
  gains execution.
* Physical Memory: The bootloader splits the total physical memory found so that each
  core only sees a small subsection of the physical memory. This physical memory range is
  then handed over to the kernel so that the kernel's allocator will only ever allocate
  memory specific to that core.
* Performance Stats: Performance stats are a way to gain introspection into the kernel.
  This is where the timing of individual pieces of the kernel will be set. The bootloader
  has full access to all core's stats. Periodically, the bootloader will accumulate the
  stats from all cores to display timing to the user.
* Alive status: A physical address to set a bit is sent to the kernel as a way to signal
  to the bootloader that this core is alive or not.

## Errchain

This bootloader/kernel leverages an `anyhow` style error handling model to enable stack
traces to be gathered in the case of an error. These errors, unlike `anyhow`, do not
rely on an allocator (more specifically `Box`), but must be `&'static` because of that.

```
use errchain::prelude::*;

pub enum Error {
    SystemTableNotFound,
}


fn table_mut() -> Result<&'static mut EfiMainSystemTable> {
    // If the table hasn't been set yet, panic since we should always have a table
    ensure!(EFI_SYSTEM_TABLE.is_some(), &Error::SystemTableNotFound);
    ...
}
```



## run*.sh

Used to test builds in qemu for `x86_64` and `aarch64`

```
run-x86_64.sh
```

```
run-aarch64.sh
```


## Build config

Reminder the bootloader is statically located at `0x2021_0000`
    
```
rustflags = ["-C", "link-arg=/debug:dwarf", 
             "-C", "link-arg=/base:0x20210000",
             "-C", "link-arg=/fixed",
             "-C", "relocation-model=static", 
             "-C", "code-model=small",
```

## Creating the custom target

```
rustc -Z unstable_options --target aarch64-pc-windows-msvc --print target-spec-json > aarch64-unknown-uefi.json
```

## Testing aarch64 on QEMU

Build this image

```
https://github.com/tianocore/edk2-platforms/tree/master/Platform/Qemu/SbsaQemu
```

Script to build the image

```
./build_aarch64_sbsa_for_qemu.sh
```
