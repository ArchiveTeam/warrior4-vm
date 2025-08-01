#!/bin/sh
# This script is downloaded and run by the warrior4-appliance service to patch
# the live system during start up of the virtual machine.
# Where this file is downloaded from is configured within the config
# file of /etc/warrior4-appliance.toml

set -e

BACKUP_TAR_PATH="/var/lib/warrior4-appliance/warrior4-backup.tar.gz"

APK_NAME="warrior4-appliance"
APK_VERSION="4.1-20250709-202543"
APK_URL="https://warriorhq.archiveteam.org/downloads/warrior4/patch/$APK_NAME-$APK_VERSION.apk"
#APK_URL="http://10.0.2.2:8000/output/$APK_NAME-$APK_VERSION.apk"
APK_SHA256="999801148ef9a42adb3a45896472aa8ac10f46ac9ac9cba508fb9cd008a05cd7"

if [ ! -f /etc/warrior4-env ]; then
    echo "This does not appear to be the warrior4 image. Exiting for safety."
    exit 1
fi

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

restart() {
    echo "The system is rebooting to apply changes"
    sleep 5
    reboot
    sleep 60
}

if [ ! -f "$BACKUP_TAR_PATH" ]; then
    backup_binaries
fi

if [ -n "$APK_SHA256" ] &&
    [ ! "$(apk info -vv | grep $APK_NAME-$APK_VERSION)" ]
then
    patch_by_apk "$APK_URL" "$APK_SHA256"
    restart
fi

echo "Done patching"
