#!/bin/sh
# Payload container pre-start script
set -e

# This directory will be a bind mount to the container's /tmp directory
mkdir -p /tmp/warrior
chmod 777 /tmp/warrior
