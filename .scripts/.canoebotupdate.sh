#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.scripts/functions.sh # import functions
source $repopath/.script/echo_colours.sh # import colours

## update pip3 modules
source $repopath/.venv/bin/activate
pip3 install -r $repopath/.scripts/requirements.txt
deactivate

git_shallow_pull $repopath
git_shallow_pull $repopath
echo_bold_red "remember to switch to the correct telegram bot!"
echo_red "defaults to botsettings.template.debug.json"
