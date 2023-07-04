#!/bin/sh
# This script is downloaded and run by the warrior4-appliance service to patch
# the live system during start up of the virtual machine

set -e

STATE_PATH="/var/lib/warrior4-appliance/patch-version"
BACKUP_TAR_PATH="/var/lib/warrior4-appliance/warrior4-backup.tar.gz"
TARBALL_STATE_PATH="/var/lib/warrior4-appliance/patch-tarball-version"

PATCH_TARBALL_URL="https://warriorhq.archiveteam.org/downloads/warrior4/patch/warrior4-patch-20000101-000000.tar.gz"
PATCH_TARBALL_SHA256=""

EXPERIMENTAL_PATCH_TARBALL_URL="http://10.0.0.2:8000/output/warrior4-patch-20000101-000000.tar.gz"
EXPERIMENTAL_PATCH_TARBALL_SHA256=""

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
system_tarball_version=""

if [ -f "$STATE_PATH" ]; then
    system_version=$(cat $STATE_PATH)
fi
if [ -f "$TARBALL_STATE_PATH" ]; then
    system_tarball_version=$(cat $TARBALL_STATE_PATH)
fi

echo "System patch version number: $system_version"
echo "System tarball version: $system_tarball_version"
echo "Allow experimental: $allow_experimental"

backup_binaries() {
    echo "Backing up binaries"

    tar -c -z -f "$BACKUP_TAR_PATH" -C / \
        /usr/bin/warrior4* \
        /usr/lib/warrior4*

    echo "Binaries backup done"
}

patch_by_tarball() {
    echo "Downloading tarball..."
    # This is BusyBox's wget
    wget "$1" -O /tmp/warrior4-patch.tar.gz
    echo "$2 /tmp/warrior4-patch.tar.gz" > /tmp/warrior4-patch-checksum
    sha256sum -c /tmp/warrior4-patch-checksum
    exit_code=$?

    if [ $exit_code = -ne 0 ]; then
        echo "Bad checksum"
        exit 1
    fi

    echo "Applying tarball"
    tar -x -z -f /tmp/warrior4-patch.tar.gz -C /

    echo "$2" > $TARBALL_STATE_PATH

    echo "Patching by tarball done"
}

if [ ! -f "$BACKUP_TAR_PATH" ]; then
    backup_binaries
fi

if [ $allow_experimental = true ] &&
    [ -n "$EXPERIMENTAL_PATCH_TARBALL_SHA256" ] &&
    [ "$EXPERIMENTAL_PATCH_TARBALL_SHA256" != "$system_tarball_version"]
then
    patch_by_tarball "$EXPERIMENTAL_PATCH_TARBALL_URL" "$EXPERIMENTAL_PATCH_TARBALL_SHA256"

elif [ -n "$PATCH_TARBALL_SHA256" ] &&
    [ "$PATCH_TARBALL_SHA256" != "$system_tarball_version" ]
then
    patch_by_tarball "$PATCH_TARBALL_URL" "$PATCH_TARBALL_SHA256"
fi

echo "Done patching"
