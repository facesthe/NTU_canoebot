#!/bin/bash
# Contains functions that are used by install.sh

# Installs itself into the home directory
# Needs to be defined and called in the target file
# turns out by using source [this_script]
# and then running it, bash installs into home
install_to_home() {
    cp $(dirname "$0")/$(basename "$0") ~
    echo $(basename "$0") installed into home directory $(realpath ~)
}

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

# Adds line to ~/.bash_aliases, if line does not exist
# Param $1: Line to add
add_bash_alias() {
    append_if_missing "$1" ~/.bash_aliases
    source ~/.bashrc
}

# Removes line to ~/.bash_aliases, if line exists
# Param $1: Line to remove
rm_bash_alias() {
    remove_matching_line "$1" ~/.bash_aliases
    source ~/.bashrc
}

# Does a shallow clone (depth 2) of the target repo
# Param $1: username
# Param $2: repo name
# Param $3: Access token (optional, for private repos)
git_shallow_clone() {
    case $# in
        0)
            echo "Shallow clone of repo."
            echo "Usage:"
            echo "  Param 1: username"
            echo "  Param 2: repo name"
            echo "  Param 3: access token (optional, for private repos)"
            ;;
        2)
            git clone --depth 2 https://"$1"@github.com/"$1"/"$2".git
            ;;
        3)
            git clone --depth 2 https://"$1":"$3"@github.com/"$1"/"$2".git
            ;;
        *)
            echo Invalid number of arguments
            ;;
    esac
}

# Does a shallow pull (depth 2) of the target repo
# OVERWRITES ALL UNCOMMITED CHANGES
# Param $1: Path to repo (optional)
git_shallow_pull() {
    case $# in
        0)
            git reset --hard origin/main
            git pull --depth 2
            ;;
        1)
            cd $1
            git reset --hard origin/main
            git pull --depth 2
            cd -
            ;;
        *)
            echo invalid arguments
            ;;
    esac
}
