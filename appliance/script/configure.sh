#!/bin/sh
# This script is run by the Alpine image creator script at the end of the
# install process.

set -e
set -x

if [ ! -f /etc/warrior4-env ]; then
    echo "This does not appear to be the warrior4 image. Exiting for safety."
    exit 1
fi

# Network
ln -s networking /etc/init.d/net.lo
ln -s networking /etc/init.d/net.eth0
rc-update add net.eth0 default
rc-update add net.lo boot

# Core system services
rc-update add acpid default
rc-update add crond default

# More system services
rc-update add consolefont boot
rc-update add chronyd default
rc-update add dnscrypt-proxy default

# Customizations
echo -e "archiveteam\narchiveteam" | passwd root

rc-update add docker default
rc-update add open-vm-tools boot

chmod 755 /etc/init.d/warrior4-appliance
chmod 755 /etc/init.d/warrior4-appliance-display
chmod 755 /usr/bin/warrior4-display-logs
chmod 755 /usr/lib/warrior4-appliance/*.sh
rc-update add warrior4-appliance default
rc-update add warrior4-appliance-display default

# Disable the console screensaver
sed -i -r 's/(default_kernel_opts="[^"]+)/\1 consoleblank=0/' /etc/update-extlinux.conf
update-extlinux
