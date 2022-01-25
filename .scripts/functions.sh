#!/bin/bash
# Contains functions that are used by install.sh

# Appends line to file if line does not exist
# Creates new file if file does not exist
# Param $1: Line to append
# Param $2: Target file
append_if_missing() {
    grep -xqFs -- "$1" "$2" || echo "$1" >> "$2"
}

# Removes line from file if line matches
# Param $1: Line to remove
# Param $2: Target file
remove_matching_line() {
    grep -vxF "$1" "$2" > .temp
    mv .temp "$2"
    # rm -f .test # not needed
}

# Adds line to cron, if line does not exist
# Param $1: Line to add
add_crontab() {
    crontab -l > .tempcron
    append_if_missing "$1" .tempcron
    crontab .tempcron
    rm -f .tempcron
}

# Removes line to cron, if line exists
# Param $1: Line to remove
rm_crontab() {
    crontab -l > .tempcron
    remove_matching_line "$1" .tempcron
    crontab .tempcron
    rm -f .tempcron
}
