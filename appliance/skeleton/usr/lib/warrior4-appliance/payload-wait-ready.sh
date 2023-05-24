#!/bin/sh
# Payload container wait for ready to access script
# Copied from https://github.com/ArchiveTeam/Ubuntu-Warrior/blob/develop/startup.sh
for i in `seq 60`; do
    sleep 5

    if docker top warrior | grep run-warrior; then
        break
    elif [ $i -eq 60 ]; then
        exit 1
    fi
done
