#!/bin/sh
# Custom fstrim scheduled job
# This file is for Alpine/BusyBox's crond setup.
# Runs BusyBox's version of fstrim and ignores errors (i.e., "not supported")
fstrim -v / || true
