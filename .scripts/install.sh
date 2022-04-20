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
source $repopath/.scripts/echo_colours.sh # colour to console

# getting some tput colours
# black=$(tput setaf 0)
# red=$(tput setaf 1)
# green=$(tput setaf 2)
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
echo_green "updating repopath..."
rm -f $repopath/.scripts/.repopath.sh
touch $repopath/.scripts/.repopath.sh
append_if_missing "# Auto-generated file. Do not modify!" $repopath/.scripts/.repopath.sh
append_if_missing "repopath='$repopath'" $repopath/.scripts/.repopath.sh

# Installing crontabs and bash aliases
for cronline in "${CRON[@]}"
do
    # echo "$green""adding crontab:$rst ${cronline:0:30} ..."
    echo_green "adding crontab:" -n
    echo "${cronline:0:30}"
    add_crontab "$cronline"
done

for aliasline in "${ALIASES[@]}"
do
    # echo "$green""adding alias:$rst ${aliasline:0:30} ..."
    echo_green "adding alias:" -n
    echo "${aliasline:0:30}"
    add_bash_alias "$aliasline"
done

echo_green "updating bash_aliases..."
source ~/.bash_aliases

# create virtual environment
echo_green "creating python3 virtual environent (venv)..."
python3 -m venv $repopath/.venv
echo_green "virtual environment created. Entering venv..."
source $repopath/.venv/bin/activate

# install pip3 packages
echo_green "installing python3 dependencies..."
pip3 install -r $repopath/.scripts/requirements.txt

echo_green "please execute 'source ~/.bash_aliases' to reload installed aliases."

## exit venv
deactivate
