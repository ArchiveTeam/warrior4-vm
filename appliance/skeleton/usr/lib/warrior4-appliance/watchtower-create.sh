#!/bin/sh
# Watchtower is configured to check for updates every hour, and to delete outdated images.
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
set -e
docker create --name watchtower -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --cleanup --include-stopped --interval 3600
