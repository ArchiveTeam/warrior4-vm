#!/bin/sh
# Payload container for warrior project web interface
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
set -e

# Create a blank configuration file if none exists, otherwise do nothing
touch /root/config.json # https://unix.stackexchange.com/a/343558

# Make sure the container has access to the config file
chmod 777 /root/config.json

# Additionally, the /root/config.json file is mounted inside the Docker container at
# /home/warrior/projects/config.json, allowing user configuration to be persisted across
# container deletions and Watchtower updates.
docker create -p 8001:8001 --name warrior \
    -v /root/config.json:/home/warrior/projects/config.json \
    --tmpfs /tmp:size=512m,mode=1777 \
    atdr.meo.ws/archiveteam/warrior-dockerfile

# Note: docker exec can't be run here because the container is not started
# Use payload-post-start.sh
