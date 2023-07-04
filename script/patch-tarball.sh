#!/bin/sh
# Compile programs and build a patch tarball

set -e
. "$(dirname "$(which "$0")")/etc.sh"

STAGING_DIR="$OUTPUT_DIR/patch-staging"
TARBALL_FILENAME=warrior4-patch-$( date --utc +%Y%m%d-%H%M%S ).tar.gz

echo "Staging directory: $STAGING_DIR"
mkdir -p $STAGING_DIR
mkdir -p $STAGING_DIR/usr/bin
mkdir -p $STAGING_DIR/usr/lib/warrior4-appliance

echo "Building binaries"
cargo build --release --target x86_64-unknown-linux-musl

echo "Copying binaries to staging directory"
install --preserve-timestamps --mode=755 \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance-display \
    $STAGING_DIR/usr/bin/

echo "Copying skeleton files to staging directory"
install --preserve-timestamps --mode=755 \
    appliance/skeleton/usr/bin/* \
    $STAGING_DIR/usr/bin/
install --preserve-timestamps --mode=755 \
    appliance/skeleton/usr/lib/warrior4-appliance/*.sh \
    $STAGING_DIR/usr/lib/warrior4-appliance/

echo "Creating tarball"
tar --create --file $OUTPUT_DIR/$TARBALL_FILENAME --directory $STAGING_DIR usr

checksum=$( sha256sum $OUTPUT_DIR/$TARBALL_FILENAME)

echo "Creating patch done"
echo "sha256sum: $checksum"
