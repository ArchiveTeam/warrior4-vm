#!/bin/sh
# Compile programs and build an apk file

set -e
. "$(dirname "$(which "$0")")/etc.sh"

STAGING_DIR="$OUTPUT_DIR/patch-staging"

# https://wiki.alpinelinux.org/wiki/Package_policies
APK_PACKAGE=warrior4-appliance
APK_VERSION="$APP_VERSION-$( date --utc +%Y%m%d-%H%M%S )"
APK_FILENAME="warrior4-appliance-$APK_VERSION.apk"

echo "Staging directory: $STAGING_DIR"
mkdir -p $STAGING_DIR
mkdir -p $STAGING_DIR/etc
mkdir -p $STAGING_DIR/etc/init.d
mkdir -p $STAGING_DIR/usr/bin
mkdir -p $STAGING_DIR/usr/lib/warrior4-appliance

echo "Building binaries"
cargo build --release --target x86_64-unknown-linux-musl

echo "Copying binaries to staging directory"
install --preserve-timestamps --mode=755 --verbose \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance \
    target/x86_64-unknown-linux-musl/release/warrior4-appliance-display \
    $STAGING_DIR/usr/bin/

echo "Copying skeleton files to staging directory"
install --preserve-timestamps --mode=755 --verbose \
    appliance/skeleton/etc/warrior4* \
    $STAGING_DIR/etc/
install --preserve-timestamps --mode=755 --verbose \
    appliance/skeleton/etc/init.d/warrior4* \
    $STAGING_DIR/etc/init.d/
install --preserve-timestamps --mode=755 --verbose \
    appliance/skeleton/usr/bin/warrior4* \
    $STAGING_DIR/usr/bin/
install --preserve-timestamps --mode=755 --verbose \
    appliance/skeleton/usr/lib/warrior4-appliance/*.sh \
    $STAGING_DIR/usr/lib/warrior4-appliance/

echo "Creating apk"
fpm -s dir -t apk -p $OUTPUT_DIR/$APK_FILENAME \
    --name $APK_PACKAGE --version $APK_VERSION \
    --chdir $STAGING_DIR \
    --config-files etc/ \
    .

checksum=$( sha256sum $OUTPUT_DIR/$APK_FILENAME)

echo "Creating patch/apk done"
echo "path: $OUTPUT_DIR/$APK_FILENAME"
echo "name: $APK_PACKAGE"
echo "version: $APK_VERSION"
echo "sha256sum: $checksum"
