#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)

## enter venv
source $repopath/.venv/bin/activate
source $repopath/.scripts/echo_colours.sh

sudo kill -15 $(pgrep -f "python3 canoebot.py")
echo_bold_red "canoebot stopped"
deactivate
