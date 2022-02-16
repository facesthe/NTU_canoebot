#!/bin/bash

source functions.sh # import functions
source .repopath.sh # import repopath

git_shallow_pull $repopath
echo "remember to edit settings.py to load botsettings.json!"
echo "defaults to botsettings.debug.json"
