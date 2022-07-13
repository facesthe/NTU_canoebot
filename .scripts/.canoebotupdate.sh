#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.scripts/functions.sh # import functions

red=$(tput setaf 1)
bold=$(tput bold)
rst=$(tput sgr0)

## update pip3 modules
source $repopath/.venv/bin/activate
pip3 install -r $repopath/.scripts/requirements.txt
deactivate

git_shallow_pull $repopath
git_shallow_pull $repopath
echo "$red""$bold""remember to switch to the correct telegram bot!""$rst"
echo "$red""defaults to botsettings.template.debug.json""$rst"
