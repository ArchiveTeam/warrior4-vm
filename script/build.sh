#!/bin/sh
# Compile programs and build the disk image

set -e
. "$(dirname "$(which "$0")")/etc.sh"

LIB_DIR="$(dirname "$(which "$0")")/../lib"

cat ./appliance/packages.txt > /dev/null
PACKAGES="$(cat ./appliance/packages.txt)"

echo "Output directory: $OUTPUT_DIR"
mkdir -p $OUTPUT_DIR

echo "Building binaries"
cargo build --release --target x86_64-unknown-linux-musl

echo "Copying binaries to skeleton directory"
install --preserve-timestamps --mode=755 \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance-display \
    appliance/skeleton/usr/bin/

echo "Creating Alpine disk image"
sudo $LIB_DIR/alpine-make-vm-image/alpine-make-vm-image \
    --image-format qcow2 \
    --image-size 60G \
    --fs-skel-dir ./appliance/skeleton \
    --fs-skel-chown root:root \
    --packages "$PACKAGES" \
    --script-chroot \
    "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" \
    -- ./appliance/script/configure.sh

echo "Converting qcow2 to vdi"
qemu-img convert -O vdi "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" "$OUTPUT_DIR/$VDI_DISK_FILENAME"

echo "Build done"
