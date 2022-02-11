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
bold=$(tput bold)
rst=$(tput sgr0)

# stores the absolute repo path ./.repopath.sh
# removes the old version with updated version
echo "$green""updating repopath...$rst"
rm -f $repopath/.scripts/.repopath.sh
touch $repopath/.scripts/.repopath.sh
append_if_missing "# Auto-generated file. Do not modify!" $repopath/.scripts/.repopath.sh
append_if_missing "repopath='$repopath'" $repopath/.scripts/.repopath.sh

# Note that for raw text to be written (no command substitution within $(this bracket) ),
# A string will need to be enclosed in 'single quotes'
# For all shell expansions, use "double quotes"
# Note that bash will read 'adjacent'"strings" as one if on the same line
CRON=(
    "@reboot sleep 30 && python3 $repopath/canoebot.py" # start bot on boot
    '5 0 * * 0 sudo kill $(pgrep python3)'"&& cd $repopath && python3 canoebot.py" # wkly restart
) # crontab entries to append

ALIASES=(
    "# read STDOUT of canoebot.py, for debugging or logging"
    "alias canoebotlog='bash $repopath/.scripts/.canoebotlog.sh'" # open stdout log
    "alias canoebotrestart='bash $repopath/.scripts/.restartbot.sh'" # restart bot
    "# end canoebot aliases #"
) # bash_aliases to append

for cronline in "${CRON[@]}"
do
    echo "$green""adding crontab:$rst $cronline"
    add_crontab "$cronline"
done

for aliasline in "${ALIASES[@]}"
do
    echo "$green""adding alias:$rst $aliasline"
    add_bash_alias "$aliasline"
done

# install pip3 packages
echo "$green""installing python3 dependencies...$rst"
# pip3 install -r $repopath/.scripts/requirements.txt
