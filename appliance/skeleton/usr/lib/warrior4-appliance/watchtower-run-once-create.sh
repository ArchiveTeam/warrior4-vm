#!/bin/sh
# Create a container so updates can be checked for on startup
# Note that this container name does not contain the string "watchtower" to avoid inaccurate grep results.
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
# Because the main Watchtower will detect this instance and refuse to run,
# we have --rm to delete this container after this runs.
set -e
docker create --name watch-once-tower --rm -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --cleanup --include-stopped --run-once
