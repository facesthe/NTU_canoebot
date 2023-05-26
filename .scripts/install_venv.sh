#!/bin/bash

# get the absolute repo path (not the .scripts path!)
# return to original path (wherever script was called)
currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

# source $repopath/.scripts/functions.sh # import functions
# source $repopath/.scripts/data.sh # cron and aliases stored here
source $repopath/.scripts/echo_colours.sh # colour to console

# create virtual environment
echo_green "creating python3 virtual environent (venv)..."
python3 -m venv $repopath/.venv
echo_green "virtual environment created. Entering venv..."
source $repopath/.venv/bin/activate

# install pip3 packages
echo_green "installing/updating python3 dependencies..."
pip3 install -r $repopath/.scripts/requirements.txt --upgrade

## exit venv
deactivate
