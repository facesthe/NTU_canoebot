#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)

## enter venv
source $repopath/.venv/bin/activate

sudo kill $(pgrep -f "python3 canoebot.py")
echo "canoebot stopped"
deactivate
