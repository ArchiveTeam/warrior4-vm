#!/bin/sh
# This script is downloaded and run by the warrior4-appliance service to patch
# the live system during start up of the virtual machine

set -e

STATE_PATH="/var/lib/warrior4-appliance/patch-version"
BACKUP_TAR_PATH="/var/lib/warrior4-appliance/warrior4-backup.tar.gz"

APK_NAME="warrior4-appliance"
APK_VERSION="4.0-20000101-000000"
APK_URL="https://warriorhq.archiveteam.org/downloads/warrior4/patch/$APK_NAME-$APK_VERSION.apk"
APK_SHA256=""

EXPERIMENTAL_APK_NAME="warrior4-appliance"
EXPERIMENTAL_APK_VERSION="4.0-20240101-120100"
EXPERIMENTAL_APK_URL="http://10.0.2.2:8000/output/$EXPERIMENTAL_APK_NAME-$EXPERIMENTAL_APK_VERSION.apk"
EXPERIMENTAL_APK_SHA256=""

if [ ! -f /etc/warrior4-env ]; then
    echo "This does not appear to be the warrior4 image. Exiting for safety."
    exit 1
fi

if [ -f /etc/warrior4-patch-experimental ]; then
    allow_experimental=true
else
    allow_experimental=false
fi

system_version=0

if [ -f "$STATE_PATH" ]; then
    system_version=$(cat $STATE_PATH)
fi

echo "System patch version number: $system_version"
echo "Allow experimental: $allow_experimental"

backup_binaries() {
    echo "Backing up binaries"

    tar -c -z -f "$BACKUP_TAR_PATH" -C / \
        /usr/bin/warrior4* \
        /usr/lib/warrior4*

    echo "Binaries backup done"
}

patch_by_apk() {
    echo "Downloading apk..."
    # This is BusyBox's wget
    wget "$1" -O /tmp/warrior4-patch.apk
    echo "$2 /tmp/warrior4-patch.apk" > /tmp/warrior4-patch-checksum
    sha256sum -c /tmp/warrior4-patch-checksum
    exit_code=$?

    if [ $exit_code -ne 0 ]; then
        echo "Bad checksum"
        exit 1
    fi

    echo "Applying apk"
    apk add --allow-untrusted /tmp/warrior4-patch.apk

    echo "Patching by apk done"
}

if [ ! -f "$BACKUP_TAR_PATH" ]; then
    backup_binaries
fi

if [ $allow_experimental = true ] &&
    [ -n "$EXPERIMENTAL_APK_SHA256" ] &&
    [ ! "$(apk info -vv | grep $EXPERIMENTAL_APK_NAME-$EXPERIMENTAL_APK_VERSION)" ]
then
    patch_by_apk "$EXPERIMENTAL_APK_URL" "$EXPERIMENTAL_APK_SHA256"

elif [ -n "$PATCH_APK_SHA256" ] &&
    [ ! "$(apk info -vv | grep $APK_NAME-$APK_VERSION)" ]
then
    patch_by_apk "$APK_URL" "$APK_SHA256"
fi

echo "Done patching"
