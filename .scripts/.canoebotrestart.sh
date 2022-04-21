#!/bin/bash

# these 4 lines get the calling path and the repo path
currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.venv/bin/activate

sudo kill -15 $(pgrep -f "python3 canoebot.py") > /dev/null
cd $repopath
nohup python3 canoebot.py >> ./.scripts/canoebot.log &
sleep 1
echo
cd $currpath
deactivate
