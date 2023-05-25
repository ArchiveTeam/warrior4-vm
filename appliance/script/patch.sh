#!/bin/sh
# This script is downloaded and run by the warrior4-appliance service to patch
# the live system during start up of the virtual machine

set -e
set -x

if [ ! -f /etc/warrior4-env ]; then
    echo "This does not appear to be the warrior4 image. Exiting for safety."
    exit 1
fi

STATE_PATH="/var/lib/warrior4-appliance/patch-version"
system_version=0

if [ -f /etc/warrior4-env ]; then
    system_version=$(< $STATE_PATH)
fi

echo "System version number: $system_version"

echo "Done patching"
