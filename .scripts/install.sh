#!/bin/bash
# Install script for FRESH systems.
# For system with the bot already installed use update.sh

# get the absolute repo path (not the .scripts path!)
# return to original path (wherever script was called)
currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.scripts/functions.sh # import functions
source $repopath/.scripts/data.sh # cron and aliases stored here

# getting some tput colours
# black=$(tput setaf 0)
# red=$(tput setaf 1)
green=$(tput setaf 2)
# blue=$(tput setaf 4)
# magenta=$(tput setaf 5)
# cyan=$(tput setaf 6)

# bg_green=$(tput setab 2)
# bg_white=$(tput setab 7)

# extras
green=$(tput setaf 2)
bold=$(tput bold)
rst=$(tput sgr0)

# stores the absolute repo path ./.repopath.sh
# removes the old version with updated version
echo "$green""updating repopath...$rst"
rm -f $repopath/.scripts/.repopath.sh
touch $repopath/.scripts/.repopath.sh
append_if_missing "# Auto-generated file. Do not modify!" $repopath/.scripts/.repopath.sh
append_if_missing "repopath='$repopath'" $repopath/.scripts/.repopath.sh

# Installing crontabs and bash aliases
for cronline in "${CRON[@]}"
do
    echo "$green""adding crontab:$rst ${cronline:0:30} ..."
    add_crontab "$cronline"
done

for aliasline in "${ALIASES[@]}"
do
    echo "$green""adding alias:$rst ${aliasline:0:30} ..."
    add_bash_alias "$aliasline"
done

echo "$green""updating bash_aliaes...$rst"
source ~/.bash_aliases

# install pip3 packages
echo "$green""installing python3 dependencies...$rst"
pip3 install -r $repopath/.scripts/requirements.txt
