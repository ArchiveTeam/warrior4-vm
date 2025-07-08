#!/bin/sh
# Compile programs and build the disk image

set -e
. "$(dirname "$(which "$0")")/etc.sh"

LIB_DIR="$(dirname "$(which "$0")")/../lib"
STAGING_DIR=$OUTPUT_DIR/staging

cat ./appliance/packages.txt > /dev/null
PACKAGES="$(cat ./appliance/packages.txt)"

echo "Output directory: $OUTPUT_DIR"
mkdir -p $OUTPUT_DIR

echo "Staging directory: $STAGING_DIR"
mkdir -p $STAGING_DIR
mkdir -p $STAGING_DIR/usr/bin/

echo "Building binaries"
cargo build --release --target x86_64-unknown-linux-musl

echo "Copying binaries to staging directory"
install --preserve-timestamps --mode=755 --verbose -D \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance-display \
    $STAGING_DIR/usr/bin/

echo "Copying skeleton files to staging directory"
(cd appliance/skeleton/ && find -type f -exec install --preserve-timestamps --mode=755 --verbose -D "{}" "../../$STAGING_DIR/{}" \;)

echo "Creating Alpine disk image"
sudo $LIB_DIR/alpine-make-vm-image/alpine-make-vm-image \
    --image-format qcow2 \
    --image-size 60G \
    --fs-skel-dir $STAGING_DIR \
    --fs-skel-chown root:root \
    --packages "$PACKAGES" \
    --script-chroot \
    "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" \
    -- ./appliance/script/configure.sh

echo "Converting qcow2 to vdi"
qemu-img convert -O vdi "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" "$OUTPUT_DIR/$VDI_DISK_FILENAME"

echo "Converting qcow2 to vmdk"
qemu-img convert -O vmdk "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" "$OUTPUT_DIR/$VMDK_DISK_FILENAME"

echo "Converting qcow2 to vhdx"
qemu-img convert -O vhdx "$OUTPUT_DIR/$QCOW2_DISK_FILENAME" "$OUTPUT_DIR/$VHDX_DISK_FILENAME"

echo "Build done"
