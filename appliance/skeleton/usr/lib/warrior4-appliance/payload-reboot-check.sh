#!/bin/sh
# Payload container that checks if a reboot is required
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh

# Can't use docker exec because the container may be exited
# docker exec warrior test -f /tmp/warrior_reboot_required
docker cp warrior:/tmp/warrior_reboot_required /tmp/warrior_reboot_required
exit $?
