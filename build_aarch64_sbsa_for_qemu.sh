#!/bin/sh

# Install prereqs
sudo apt install uuid-dev gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu iasl

# Create the aarch64 build dir
mkdir aarch64_build

# Go into the aarch64 build dir
cd aarch64_build

# Setup the workspace env variables
export WORKSPACE=`pwd`
export PACKAGES_PATH=$WORKSPACE/edk2:$WORKSPACE/edk2-platforms:$WORKSPACE/edk2-non-osi
export GCC5_AARCH64_PREFIX=aarch64-linux-gnu-

# Build and compile qemu 4.1+ for Sbsa-ref support
git clone --depth 1 --recursive https://github.com/qemu/qemu
cd qemu
../configure --target-list=aarch64-softmmu --prefix=$WORKSPACE
make install

# Download prereq repos
git clone --depth 1 --recursive https://github.com/tianocore/edk2
git clone --depth 1 --recursive https://github.com/tianocore/edk2-platforms
git clone --depth 1 --recursive https://github.com/tianocore/edk2-non-osi
git clone --depth 1 --recursive https://git.trustedfirmware.org/TF-A/trusted-firmware-a

# Build base tools as per the README
# https://github.com/tianocore/edk2-platforms/tree/master/Platform/Qemu/SbsaQemu
make -C edk2/BaseTools
. edk2/edksetup.sh

# Build the image
build -b RELEASE -a AARCH64 -t GCC5 -p edk2-platforms/Platform/Qemu/SbsaQemu/SbsaQemu.dsc

# Copy and truncate the build images
cp Build/SbsaQemu/RELEASE_GCC5/FV/SBSA_FLASH[01].fd .
truncate -s 256M SBSA_FLASH[01].fd

# Delete the old directories
rm -rf Build edk2 edk2-non-osi edk2-platforms trusted-firmware-a
