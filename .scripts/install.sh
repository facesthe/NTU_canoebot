#!/bin/bash
# Install script for FRESH system.
# For system with the bot already installed use update.sh

# Param $1: Line to append
# Param $2: Target file
append_if_missing() {
    grep -qxF 'include "$1"' $2 ||\
     echo 'include "/configs/projectname.conf"' >> $2
}