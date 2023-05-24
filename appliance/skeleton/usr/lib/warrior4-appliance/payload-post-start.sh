#!/bin/sh
# Payload container post-start script
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
set -e

# Allow reading network stats by non-root
# Run the adduser command as root: https://stackoverflow.com/a/35485346
docker exec -u 0 warrior adduser warrior dip

docker exec warrior \
    rm -f /tmp/warrior_reboot_required \
    /tmp/warrior_poweroff_required
