#!/bin/sh
# Create a container so updates can be checked for on startup
# Note that this container name does not contain the string "watchtower" to avoid inaccurate grep results.
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
set -e
docker create --name watch-once-tower -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --cleanup --include-stopped --run-once
